use std::{
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::{Either, OptionFuture},
    pin_mut, ready,
    stream::{self, Fuse},
    FutureExt, Sink, SinkExt, Stream, StreamExt,
};
use pin_project_lite::pin_project;
use serde_json::Value;
use streamunordered::{StreamUnordered, StreamYield};

use super::{AsyncRuntime, ExecutorResult, IncomingMessage, Request, Requests, Response, Task};
use crate::{
    internal::{
        exec::{self, ResponseInner},
        exec2, PinnedOption, PinnedOptionProj,
    },
    Router,
};

// Time to wait for more messages before sending them over the websocket connection.
// This batch is mostly designed to reduce the impact of duplicate subscriptions a bit
// as sending them together should help us utilise transport layer compression.
const BATCH_TIMEOUT: Duration = Duration::from_millis(5);

enum PollResult {
    /// The poller has done some progressed work.
    /// WARNING: this does not guarantee any wakers have been registered so to uphold the `Future` invariants you can not return.
    Progressed,

    /// The poller has queued a message to be sent.
    /// WARNING: You must call `Self::poll_send` to prior to returning from the `Future::poll` method.
    QueueSend,

    /// The future is complete
    Complete,
}

// struct ConnectionSubscriptionManager<'a, TCtx> {
//     pub map: &'a mut SubscriptionSet,
//     pub to_abort: Option<Vec<u32>>,
//     pub queued: Option<Vec<RspcTask<TCtx>>>,
// }

// impl<'a, TCtx: Clone + Send + 'static> SubscriptionManager<TCtx>
//     for ConnectionSubscriptionManager<'a, TCtx>
// {
//     type Set<'m> = &'m mut SubscriptionSet where Self: 'm;

//     fn queue(&mut self, stream: RspcTask<TCtx>) {
//         match &mut self.queued {
//             Some(queued) => {
//                 queued.push(stream);
//             }
//             None => self.queued = Some(vec![stream]),
//         }
//     }

//     fn subscriptions(&mut self) -> Self::Set<'_> {
//         self.map
//     }

//     fn abort_subscription(&mut self, id: u32) {
//         self.to_abort.get_or_insert_with(Vec::new).push(id);
//     }
// }

// type MyTask = Either<Once<RequestTask>, ()>; // TODO: This requires `RequestTask` to be public and not sealed which I am not the biggest fan of?

struct Connection<TCtx> {
    ctx: TCtx,
    router: Arc<Router<TCtx>>,
    conn: exec2::Connection,

    streams: StreamUnordered<Task>,

    // TODO: Remove these cause disgusting messes
    sub_id_to_stream: HashMap<u32, usize>,
}

impl<TCtx> Connection<TCtx>
where
    TCtx: Clone + Send + 'static,
{
    pub fn exec(&mut self, reqs: Vec<Request>) -> Vec<Response> {
        let mut resps = Vec::with_capacity(reqs.len());
        for req in reqs {
            match self
                .router
                .execute(self.ctx.clone(), req, Some(&mut self.conn))
            {
                ExecutorResult::Task(task) => {
                    let fut_id = task.id;
                    let token = self.streams.insert(task);
                    self.sub_id_to_stream.insert(fut_id, token);
                }
                ExecutorResult::Future(fut) => {
                    let fut_id = fut.id;
                    let token = self.streams.insert(fut.into());
                    self.sub_id_to_stream.insert(fut_id, token);
                }
                ExecutorResult::Response(resp) => {
                    resps.push(resp);
                }
                ExecutorResult::None => {}
            }
        }

        // TODO: Fix all of this!
        // let manager = manager.expect("rspc unreachable");
        // if let Some(to_abort) = manager.to_abort {
        //     for sub_id in to_abort {
        //         if let Some(token) = self.sub_id_to_stream.remove(&sub_id) {
        //             Pin::new(&mut self.streams).remove(token);
        //             manager.map.take(&sub_id);
        //         }
        //     }
        // }

        // TODO: Fix all of this!
        // if let Some(queued) = manager.queued {
        //     for stream in queued {
        //         let sub_id = stream.id();
        //         let token = self.streams.insert(stream);
        //         self.sub_id_to_stream.insert(sub_id, token);
        //     }
        // }
        // todo!();

        resps
    }
}

macro_rules! unwrap_return {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return,
        }
    };
}

fn batch_unbounded<R: AsyncRuntime, T>(
    (tx, mut rx): (UnboundedSender<T>, UnboundedReceiver<T>),
) -> (UnboundedSender<T>, stream::Fuse<impl Stream<Item = Vec<T>>>) {
    (
        tx,
        async_stream::stream! {
            loop {
                let mut responses = vec![unwrap_return!(rx.next().await)];

                'batch: loop {
                    let timer = R::sleep_util(Instant::now() + BATCH_TIMEOUT).fuse();

                    'timer:  loop {
                        pin_mut!(timer);

                        futures::select_biased! {
                            response = rx.next() => {
                                responses.push(unwrap_return!(response));
                                break 'timer;
                            }
                            _ = timer => break 'batch,
                        }
                    };
                }

                yield responses;
            }
        }
        .fuse(),
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
    S: Sink<Vec<Response>, Error = E> + Stream<Item = Result<IncomingMessage, E>> + Send + Unpin,
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
        conn: exec2::Connection::new(),
        streams: StreamUnordered::new(),
        sub_id_to_stream: HashMap::new(),
    };

    let (batch_tx, batch_stream) = batch_unbounded::<R, _>(mpsc::unbounded());
    pin_mut!(batch_stream);

    let mut done = false;

    let mut socket = socket.fuse();

    loop {
        if done {
            break;
        };

        futures::select_biased! {
            recv = OptionFuture::from(clear_subscriptions_rx.as_mut().map(StreamExt::next)) => {
                if let Some(Some(())) = recv {
                    for (_, token) in conn.sub_id_to_stream.drain() {
                        Pin::new(&mut conn.streams).remove(token);
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
                        done = true;
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
                            conn.sub_id_to_stream.remove(&sub_id);
                            // conn.map.take(&sub_id);
                        }
                    }
                }
            }
        }
    }
}
