use futures::{ready, stream::SplitSink, Sink, SinkExt, Stream, StreamExt};
use httpz::{
    http::{Response, StatusCode},
    ws::{Message, Websocket, WebsocketUpgrade},
    HttpResponse,
};
use pin_project::pin_project;
use serde_json::Value;
use std::{
    borrow::Cow,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use streamunordered::{FinishedStream, StreamUnordered, StreamYield};

use crate::internal::{
    exec::{
        self, AsyncRuntime, Batcher, Connection, ExecRequestFut, Executor, ExecutorResult,
        GenericSubscriptionManager, OwnedStream, SubscriptionManager, SubscriptionMap,
        TokioRuntime, TrustMeBro,
    },
    PinnedOption, PinnedOptionProj,
};

use super::TCtxFunc;

pub(crate) fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    executor: Executor<TCtx, TokioRuntime>,
    ctx_fn: TCtxFn,
    req: httpz::Request,
) -> impl HttpResponse
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    if !req.server().supports_websockets() {
        #[cfg(feature = "tracing")]
        tracing::debug!("Websocket are not supported on your webserver!");

        // TODO: Make this error be picked up on the frontend and expose it with a logical name
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![])?);
    } else {
        #[cfg(feature = "tracing")]
        tracing::debug!("Accepting websocket connection");
    }

    // TODO: Remove need for `_internal_dangerously_clone`
    let ctx = match ctx_fn.exec(req._internal_dangerously_clone(), None) {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(vec![])?);
        }
    };

    let cookies = req.cookies(); // TODO: Reorder args of next func so cookies goes first
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |_, socket| {
        WebsocketTask::<TokioRuntime, TCtx>::new(Connection::new(ctx, executor), socket)
    })
    .into_response()
}

/// TODO
#[pin_project(project = PlzNameThisEnumProj)]
pub(crate) enum PlzNameThisEnum<TCtx: 'static> {
    OwnedStream(#[pin] OwnedStream<TCtx>),
    ExecRequestFut(#[pin] PinnedOption<ExecRequestFut>),
}

impl<TCtx: 'static> Stream for PlzNameThisEnum<TCtx> {
    type Item = exec::Response;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            PlzNameThisEnumProj::OwnedStream(s) => {
                let s = s.project();
                match s.reference.poll_next(cx) {
                    Poll::Ready(v) => Poll::Ready(v.map(|r| match r {
                        Ok(v) => exec::Response {
                            id: *s.id,
                            result: exec::ValueOrError::Value(v),
                        },
                        Err(err) => exec::Response {
                            id: *s.id,
                            result: exec::ValueOrError::Error(err.into()),
                        },
                    })),
                    Poll::Pending => Poll::Pending,
                }
            }
            PlzNameThisEnumProj::ExecRequestFut(mut s) => match s.as_mut().project() {
                PinnedOptionProj::Some(ss) => match ss.poll(cx) {
                    Poll::Ready(v) => {
                        s.set(PinnedOption::None);
                        // this.set(None);
                        Poll::Ready(Some(v))
                    }
                    Poll::Pending => Poll::Pending,
                },
                PinnedOptionProj::None => Poll::Ready(None),
            },
        }
    }
}

/// TODO
#[pin_project(project = SubscriptionThreadProj)]
pub(super) struct WebsocketTask<R: AsyncRuntime, TCtx: Clone + Send + 'static> {
    #[pin]
    conn: Connection<R, TCtx>,
    #[pin]
    batch: Batcher<R>,

    #[pin]
    socket: Box<dyn Websocket + Send>,
    tx_queue: Option<Message>,
}

impl<R: AsyncRuntime, TCtx: Clone + Send + 'static> WebsocketTask<R, TCtx> {
    pub fn new(conn: Connection<R, TCtx>, socket: Box<dyn Websocket + Send>) -> Self {
        Self {
            conn,
            batch: Batcher::new(),
            socket,
            tx_queue: None,
        }
    }
}

impl<R: AsyncRuntime, TCtx: Clone + Send + 'static> Future for WebsocketTask<R, TCtx> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();
        loop {
            // TODO: Terminate when asked to by subscription manager

            if let Some(json) = ready!(this.batch.as_mut().poll_next(cx))
                .expect("rspc: batcher stream ended unexpectedly")
            {
                *this.tx_queue = Some(Message::Text(json))
            }

            if let Some(_) = this.tx_queue {
                ready!(this.socket.as_mut().poll_ready(cx)).unwrap(); // TODO: Error handling

                let item = this.tx_queue.take().expect("rspc unreachable");
                println!("YEET {:?}", item); // TODO
                this.socket.as_mut().start_send(item).unwrap(); // TODO: Error handling
            }

            ready!(this.socket.as_mut().poll_flush(cx)).unwrap(); // TODO: Error handling

            match this.socket.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(msg))) => {
                    println!("INCOMING MSG {:?}", msg); // TODO

                    let res = match msg {
                        Message::Text(text) => serde_json::from_str::<Value>(&text),
                        Message::Binary(binary) => serde_json::from_slice(&binary),
                        Message::Ping(_) | Message::Pong(_) => continue,
                        Message::Close(_) => {
                            // TODO: Terminate all subscriptions
                            // TODO: Tell frontend all subscriptions were terminated

                            println!("SHUTDOWN"); // TODO

                            return Poll::Ready(());
                        }
                        Message::Frame(_) => unreachable!(),
                    };

                    println!("REQS {:?}", res);

                    match res.and_then(|v| match v.is_array() {
                        true => serde_json::from_value::<Vec<exec::Request>>(v),
                        false => serde_json::from_value::<exec::Request>(v).map(|v| vec![v]),
                    }) {
                        Ok(reqs) => {
                            this.batch.as_mut().append(&mut this.conn.exec(reqs));
                        }
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error parsing websocket message: {}", _err);

                            println!("ERR {:?}", _err); // TODO

                            // TODO: Send report of error to frontend

                            continue;
                        }
                    }
                }
                Poll::Ready(Some(Err(_err))) => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("Error reading from websocket connection: {:?}", _err);

                    println!("ERR"); // TODO

                    continue;
                }
                Poll::Ready(None) => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("Shutting down websocket connection");

                    // TODO: Terminate all subscriptions
                    // TODO: Tell frontend all subscriptions were terminated

                    println!("SHUTDOWN"); // TODO
                    return Poll::Ready(());
                }
                Poll::Pending => return Poll::Pending,
            }

            if let Some(batch) = ready!(this.conn.as_mut().poll_next(cx))
                .expect("rspc: connection stream ended unexpectedly")
            {
                this.batch.as_mut().insert(batch);
            }
        }
    }
}
