use std::{
    future::poll_fn,
    pin::{pin, Pin},
    sync::Arc,
    task::Poll,
};

use futures::{channel::mpsc, future::OptionFuture, FutureExt, Sink, SinkExt, Stream, StreamExt};
use streamunordered::{StreamUnordered, StreamYield};

use super::{ExecutorResult, IncomingMessage, Request, Response, SubscriptionMap, Task};
use crate::{exec, Router};

/// Constant used to determine how many times we are allowed to poll underlying futures without yielding.
///
/// We continue to poll the stream until nothing is immediately ready or we hit this limit.
/// Imagine a subscription that subscribes to an mpsc.
/// If the client creates multiple subscriptions it's fairly likely that many events may be ready at once.
/// This causes them to get batched in groups of N.
const YIELD_EVERY: usize = 25;

struct Connection<TCtx> {
    ctx: TCtx,
    router: Arc<Router<TCtx>>,
    streams: StreamUnordered<Task>,
    subscriptions: SubscriptionMap,
}

impl<TCtx> Connection<TCtx>
where
    TCtx: Clone + Send + 'static,
{
    pub fn exec(&mut self, reqs: Vec<Request>) -> Vec<Response> {
        let mut resps = Vec::with_capacity(reqs.len());

        for req in reqs {
            let Some(res) =
                self.router
                    .clone()
                    .execute(self.ctx.clone(), req, Some(&mut self.subscriptions))
            else {
                continue;
            };

            match res {
                ExecutorResult::Task(task) => {
                    let task_id = task.id;
                    self.streams.insert(task);
                    self.subscriptions.shutdown(task_id);
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
        subscriptions: Default::default(),
    };

    let mut batch: Vec<Response> = vec![];
    let mut socket = pin!(socket.fuse());

    loop {
        if !batch.is_empty() {
            let batch = batch.drain(..batch.len()).collect::<Vec<_>>();
            if let Err(_err) = socket.send(batch).await {
                #[cfg(feature = "tracing")]
                tracing::error!("Error sending message to websocket: {}", _err);
            }
        }

        futures::select_biased! {
            recv = OptionFuture::from(clear_subscriptions_rx.as_mut().map(StreamExt::next)) => {
                if let Some(Some(())) = recv {
                    conn.subscriptions.shutdown_all();
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
                                    .for_each(|resp| batch.push(resp));
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
                        batch.push(resp);

                        poll_fn(|cx| {
                            for _ in 0..YIELD_EVERY {
                                match conn.streams.select_next_some().poll_unpin(cx) {
                                    Poll::Pending => break,
                                    Poll::Ready((v, _)) => match v {
                                        StreamYield::Item(resp) => batch.push(resp),
                                        StreamYield::Finished(f) => {
                                            if let Some(stream) = f.take(Pin::new(&mut conn.streams)) {
                                                conn.subscriptions._internal_remove(stream.id);
                                            }
                                        }
                                    }
                                }
                            }

                            Poll::Ready(())
                        }).await;
                    }
                    StreamYield::Finished(f) => {
                        if let Some(stream) = f.take(Pin::new(&mut conn.streams)) {
                            conn.subscriptions._internal_remove(stream.id);
                        }
                    }
                }
            }
        }
    }
}
