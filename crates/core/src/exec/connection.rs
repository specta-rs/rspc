use std::{
    collections::HashMap,
    pin::{pin, Pin},
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::OptionFuture,
    pin_mut,
    stream::{self, FusedStream, FuturesUnordered},
    FutureExt, Sink, SinkExt, Stream, StreamExt,
};
use streamunordered::{StreamUnordered, StreamYield};

use super::{ExecutorResult, IncomingMessage, Request, Response, Task};
use crate::{exec, AsyncRuntime, Router};

// Time to wait for more messages before sending them over the websocket connection.
// This batch is mostly designed to reduce the impact of duplicate subscriptions a bit
// as sending them together should help us utilise transport layer compression.
const BATCH_TIMEOUT: Duration = Duration::from_millis(5);

// TODO: I don't like this
pub(crate) struct TaskShutdown {
    stream_id: usize,
    tx: oneshot::Sender<usize>,
}

impl TaskShutdown {
    pub fn send(self) -> Result<(), usize> {
        self.tx.send(self.stream_id)
    }
}

pub struct Connection<TCtx> {
    ctx: TCtx,
    router: Arc<Router<TCtx>>,

    streams: StreamUnordered<Task>,

    subscription_shutdown_rx: FuturesUnordered<oneshot::Receiver<usize>>,
    pub(crate) subscription_shutdowns: HashMap<u32, TaskShutdown>,
}

impl<TCtx> Connection<TCtx>
where
    TCtx: Clone + Send + 'static,
{
    pub fn exec(&mut self, reqs: Vec<Request>) -> Vec<Response> {
        let mut resps = Vec::with_capacity(reqs.len());

        for req in reqs {
            let Some(res) = self
                .router
                .clone()
                .execute(self.ctx.clone(), req, Some(self))
            else {
                continue;
            };

            match res {
                ExecutorResult::Task(task) => {
                    let task_id = task.id;
                    let stream_id = self.streams.insert(task);

                    let (tx, rx) = oneshot::channel();

                    self.subscription_shutdowns
                        .insert(task_id, TaskShutdown { stream_id, tx });
                    self.subscription_shutdown_rx.push(rx);
                }
                ExecutorResult::Future(fut) => {
                    self.streams.insert(fut.into());
                }
                ExecutorResult::Response(resp) => {
                    resps.push(resp);
                }
            }
        }

        resps
    }
}

fn batch_unbounded<R: AsyncRuntime, T>(
    (tx, rx): (UnboundedSender<T>, UnboundedReceiver<T>),
) -> (UnboundedSender<T>, impl Stream<Item = Vec<T>> + FusedStream) {
    (
        tx,
        stream::unfold(rx, |mut rx| async move {
            let mut responses = vec![rx.next().await?];

            'batch: loop {
                let timer = R::sleep_util(Instant::now() + BATCH_TIMEOUT).fuse();

                #[allow(clippy::never_loop)]
                'timer: loop {
                    pin_mut!(timer);

                    futures::select_biased! {
                        response = rx.next() => {
                            responses.push(response?);
                            break 'timer;
                        }
                        _ = timer => break 'batch,
                    }
                }
            }

            Some((responses, rx))
        }),
    )
}

/// An abstraction around a single "connection" which can execute rspc subscriptions.
///
/// For Tauri a "connection" would be for each webpage and for HTTP that is a whole websocket connection.
///
/// This future is spawned onto a thread and coordinates everything. It handles:
/// - Sending to connection
/// - Reading from connection
/// - Executing requests and subscriptions
/// - Batching responses
pub async fn run_connection<
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
    S: Sink<Vec<Response>, Error = E> + Stream<Item = Result<IncomingMessage, E>> + Send,
    E: std::fmt::Debug + std::error::Error,
>(
    ctx: TCtx,
    router: Arc<Router<TCtx>>,
    socket: S,
    mut clear_subscriptions_rx: Option<mpsc::UnboundedReceiver<()>>,
) {
    let mut conn = Connection {
        ctx,
        router,
        streams: Default::default(),
        subscription_shutdown_rx: Default::default(),
        subscription_shutdowns: Default::default(),
    };

    let (batch_tx, batch_stream) = batch_unbounded::<R, _>(mpsc::unbounded());
    pin_mut!(batch_stream);

    let mut socket = pin!(socket.fuse());

    loop {
        futures::select_biased! {
            recv = OptionFuture::from(clear_subscriptions_rx.as_mut().map(StreamExt::next)) => {
                if let Some(Some(())) = recv {
                    for (_, shutdown) in conn.subscription_shutdowns.drain() {
                        shutdown.tx.send(shutdown.stream_id).ok();
                    }
                }
            }
            responses = batch_stream.next() => {
                if let Some(responses) = responses {
                    if let Err(_err) = socket.send(responses).await {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error sending message to websocket: {}", _err);
                    }
                }
            }
            // poll_recv
            msg = socket.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        let res = match msg {
                            IncomingMessage::Msg(res) => res,
                            IncomingMessage::Close => { break },
                            IncomingMessage::Skip => { continue },
                        };

                        match res.and_then(|v| match v.is_array() {
                            true => serde_json::from_value::<Vec<exec::Request>>(v),
                            false => serde_json::from_value::<exec::Request>(v).map(|v| vec![v]),
                        }) {
                            Ok(reqs) => {
                                conn.exec(reqs)
                                    .into_iter()
                                    .for_each(|resp| {
                                        batch_tx.unbounded_send(resp).expect("Failed to send on unbounded send");
                                    });
                            }
                            Err(_err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!("Error parsing websocket message: {}", _err);

                                // TODO: Send report of error to frontend but who do we correlated them????
                            }
                        }
                    }
                    Some(Err(_err)) => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Error reading from websocket connection: {:?}", _err);

                        // TODO: Send report of error to frontend but who do we correlated them????
                    },
                    None => {
                        break
                    }
                }
            }
            // poll_streams
            value = conn.streams.select_next_some() => {
                let (yld, _) = value;

                match yld {
                    StreamYield::Item(resp) => {
                        batch_tx.unbounded_send(resp).ok();
                    }
                    StreamYield::Finished(f) => {
                        if let Some(stream) = f.take(Pin::new(&mut conn.streams)) {
                            let sub_id = stream.id;
                            conn.subscription_shutdowns.remove(&sub_id);
                        }
                    }
                }
            }
            shutdown = conn.subscription_shutdown_rx.select_next_some() => {
                if let Ok(stream_id) = shutdown {
                    Pin::new(&mut conn.streams).remove(stream_id);
                }
            }
        }
    }

    println!("Connection done!");
}
