use futures::{Sink, Stream};
use httpz::{
    http::{Response, StatusCode},
    ws::{Message, Websocket, WebsocketUpgrade},
    HttpResponse,
};
use pin_project::pin_project;
use serde_json::Value;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::internal::{
    exec::{
        self, AsyncRuntime, Batcher, Connection, ExecRequestFut, Executor, OwnedStream,
        TokioRuntime,
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
        let mut is_pending = false;

        while !is_pending {
            // TODO: Terminate when asked to by subscription manager

            if this.tx_queue.is_none() {
                match this.batch.as_mut().poll_next(cx) {
                    Poll::Ready(Some(Some(json))) => *this.tx_queue = Some(Message::Text(json)),
                    Poll::Ready(Some(None)) => {}
                    Poll::Ready(None) => panic!("rspc: batcher stream ended unexpectedly"),
                    Poll::Pending => is_pending = true,
                };
            }

            if let Some(_) = this.tx_queue {
                match this.socket.as_mut().poll_ready(cx) {
                    Poll::Ready(Ok(())) => {}
                    Poll::Ready(Err(err)) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error waiting for websocket to be ready: {}", err);
                    }
                    Poll::Pending => is_pending = true,
                }

                let item = this
                    .tx_queue
                    .take()
                    // We check it is Some every poll but defer taking it from the `Option` until the socket is ready
                    .expect("rspc unreachable as we just checked this is `Option::Some`");

                if let Err(err) = this.socket.as_mut().start_send(item) {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error sending message to websocket: {}", err);
                }
            }

            match this.socket.as_mut().poll_flush(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(err)) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error flushing message to websocket: {}", err);
                }
                Poll::Pending => is_pending = true,
            };

            loop {
                match this.socket.as_mut().poll_next(cx) {
                    Poll::Ready(Some(Ok(msg))) => {
                        let res = match msg {
                            Message::Text(text) => serde_json::from_str::<Value>(&text),
                            Message::Binary(binary) => serde_json::from_slice(&binary),
                            Message::Ping(_) | Message::Pong(_) => continue,
                            Message::Close(_) => {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("Shutting down websocket connection");
                                // TODO: Terminate all subscriptions
                                // TODO: Tell frontend all subscriptions were terminated

                                return Poll::Ready(());
                            }
                            Message::Frame(_) => unreachable!(),
                        };

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

                                // TODO: Send report of error to frontend
                            }
                        }
                    }
                    Poll::Ready(Some(Err(_err))) => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Error reading from websocket connection: {:?}", _err);

                        // TODO: Send report of error to frontend
                    }
                    Poll::Ready(None) => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Shutting down websocket connection");

                        // TODO: Terminate all subscriptions
                        // TODO: Tell frontend all subscriptions were terminated

                        return Poll::Ready(());
                    }
                    Poll::Pending => {
                        is_pending = true;
                        break;
                    }
                }
            }

            loop {
                match this.conn.as_mut().poll_next(cx) {
                    Poll::Ready(Some(batch)) => {
                        if let Some(batch) = batch {
                            this.batch.as_mut().insert(batch);
                        }
                    }
                    Poll::Ready(None) => unreachable!(),
                    Poll::Pending => {
                        is_pending = true;
                        break;
                    }
                }
            }
        }

        Poll::Pending
    }
}
