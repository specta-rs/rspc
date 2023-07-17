use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Sink, Stream};
use pin_project::pin_project;
use serde_json::Value;

use super::{AsyncRuntime, Batcher, Connection};
use crate::internal::exec;

// TODO: Maybe merge this `Connection` and `Batch` abstractions into this?
// TODO: Rewrite this to named enums instead of `Option<bool>` and the like

#[derive(Debug)]
pub(crate) enum IncomingMessage {
    Msg(Result<Value, serde_json::Error>),
    Close,
    Skip,
}

pub(crate) struct OutgoingMessage(pub String);

#[cfg(feature = "httpz")]
impl From<httpz::ws::Message> for IncomingMessage {
    fn from(value: httpz::ws::Message) -> Self {
        match value {
            httpz::ws::Message::Text(v) => Self::Msg(serde_json::from_str(&v)),
            httpz::ws::Message::Binary(v) => Self::Msg(serde_json::from_slice(&v)),
            httpz::ws::Message::Ping(_) | httpz::ws::Message::Pong(_) => Self::Skip,
            httpz::ws::Message::Close(_) => Self::Close,
            httpz::ws::Message::Frame(_) => {
                #[cfg(debug_assertions)]
                unreachable!("Reading a 'httpz::ws::Message::Frame' is impossible");

                #[cfg(not(debug_assertions))]
                return Self::Skip;
            }
        }
    }
}

#[cfg(feature = "httpz")]
impl From<OutgoingMessage> for httpz::ws::Message {
    fn from(value: OutgoingMessage) -> Self {
        Self::Text(value.0)
    }
}

/// TODO
#[pin_project(project = ConnectionTaskProj)]
pub(crate) struct ConnectionTask<
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
    S: Sink<M, Error = E> + Stream<Item = Result<M2, E>> + Send + Unpin,
    M: From<OutgoingMessage>,
    M2: Into<IncomingMessage>,
    E: std::fmt::Debug + std::error::Error,
> {
    #[pin]
    conn: Connection<R, TCtx>,
    #[pin]
    batch: Batcher<R>,

    #[pin]
    socket: S,
    tx_queue: Option<M>,

    phantom: PhantomData<M2>,
}

impl<
        R: AsyncRuntime,
        TCtx: Clone + Send + 'static,
        S: Sink<M, Error = E> + Stream<Item = Result<M2, E>> + Send + Unpin,
        M: From<OutgoingMessage>,
        M2: Into<IncomingMessage>,
        E: std::fmt::Debug + std::error::Error,
    > ConnectionTask<R, TCtx, S, M, M2, E>
{
    pub fn new(conn: Connection<R, TCtx>, socket: S) -> Self {
        Self {
            conn,
            batch: Batcher::new(),
            socket,
            tx_queue: None,
            phantom: PhantomData,
        }
    }

    /// Poll sending
    ///
    /// `Poll::Ready(())` is returned no wakers have been registered. This invariant must be maintained by caller!
    fn poll_send(
        this: &mut ConnectionTaskProj<R, TCtx, S, M, M2, E>,
        cx: &mut Context<'_>,
    ) -> Poll<()> {
        // If nothing in `tx_queue`, poll the batcher to populate it
        if this.tx_queue.is_none() {
            match ready!(this.batch.as_mut().poll_next(cx)) {
                Some(Some(json)) => *this.tx_queue = Some(OutgoingMessage(json).into()),
                Some(None) => {}
                None => panic!("rspc: batcher stream ended unexpectedly"),
            };
        }

        // If something is queued to send
        if let Some(_) = this.tx_queue {
            // Wait until the socket is ready for sending
            if let Err(err) = ready!(this.socket.as_mut().poll_ready(cx)) {
                #[cfg(feature = "tracing")]
                tracing::error!("Error waiting for websocket to be ready: {}", err);

                return Poll::Ready(());
            };

            let item = this
                .tx_queue
                .take()
                // We check it is `Some(_)` every poll but defer taking it from the `Option` until the socket is ready
                .expect("rspc unreachable");

            if let Err(err) = this.socket.as_mut().start_send(item) {
                #[cfg(feature = "tracing")]
                tracing::error!("Error sending message to websocket: {}", err);
            }
        }

        // Flush the previously sent data if any is pending
        if let Err(err) = ready!(this.socket.as_mut().poll_flush(cx)) {
            #[cfg(feature = "tracing")]
            tracing::error!("Error flushing message to websocket: {}", err);
        }

        Poll::Ready(())
    }

    /// Poll receiving
    ///
    /// `Poll::Ready(Some(true))` is returned the entire future is complete. This invariant must be maintained by caller!
    /// `Poll::Ready(Some(false))` means you must `Self::poll_send`. This invariant must be maintained by caller!
    /// `Poll::Ready(None)` is returned no wakers have been registered. This invariant must be maintained by caller!
    fn poll_recv(
        this: &mut ConnectionTaskProj<R, TCtx, S, M, M2, E>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<bool>> {
        match ready!(this.socket.as_mut().poll_next(cx)) {
            Some(Ok(msg)) => {
                let res = match msg.into() {
                    IncomingMessage::Msg(json) => json,
                    IncomingMessage::Close => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Shutting down websocket connection");

                        // TODO: Terminate all subscriptions
                        // TODO: Tell frontend all subscriptions were terminated

                        return Poll::Ready(Some(true));
                    }
                    IncomingMessage::Skip => return Poll::Ready(None),
                };

                match res.and_then(|v| match v.is_array() {
                    true => serde_json::from_value::<Vec<exec::Request>>(v),
                    false => serde_json::from_value::<exec::Request>(v).map(|v| vec![v]),
                }) {
                    Ok(reqs) => {
                        let mut a = this.conn.exec(reqs);
                        println!("C {a:?}");
                        this.batch.as_mut().append(&mut a);

                        return Poll::Ready(Some(false));
                    }
                    Err(_err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error parsing websocket message: {}", _err);

                        // TODO: Send report of error to frontend
                        println!("D {_err:?}");
                    }
                }
            }
            Some(Err(_err)) => {
                #[cfg(feature = "tracing")]
                tracing::debug!("Error reading from websocket connection: {:?}", _err);

                // TODO: Send report of error to frontend
                println!("E {_err:?}");
            }
            None => {
                #[cfg(feature = "tracing")]
                tracing::debug!("Shutting down websocket connection");

                // TODO: Terminate all subscriptions
                // TODO: Tell frontend all subscriptions were terminated

                return Poll::Ready(Some(true));
            }
        }

        Poll::Ready(None)
    }

    /// Poll active streams
    ///
    /// `Poll::Ready(false)` is returned no wakers have been registered. This invariant must be maintained by caller!
    /// `Poll::Ready(true)` means you must `Self::poll_send`. This invariant must be maintained by caller!
    fn poll_streams(
        this: &mut ConnectionTaskProj<R, TCtx, S, M, M2, E>,
        cx: &mut Context<'_>,
    ) -> Poll<bool> {
        if let Some(batch) = ready!(this.conn.as_mut().poll_next(cx)).expect("rspc unreachable") {
            this.batch.as_mut().insert(batch);
            return Poll::Ready(true);
        }

        Poll::Ready(false)
    }
}

impl<
        R: AsyncRuntime,
        TCtx: Clone + Send + 'static,
        S: Sink<M, Error = E> + Stream<Item = Result<M2, E>> + Send + Unpin,
        M: From<OutgoingMessage>,
        M2: Into<IncomingMessage>,
        E: std::fmt::Debug + std::error::Error,
    > Future for ConnectionTask<R, TCtx, S, M, M2, E>
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();
        let mut is_pending = false;

        while !is_pending {
            // TODO: Terminate when asked by subscription manager

            if let Poll::Pending = Self::poll_send(&mut this, cx) {
                is_pending = true;
            }

            match Self::poll_recv(&mut this, cx) {
                Poll::Ready(Some(true)) => return Poll::Ready(()),
                Poll::Ready(Some(false)) => continue,
                Poll::Ready(None) => {}
                Poll::Pending => {
                    is_pending = true;
                }
            }

            match Self::poll_streams(&mut this, cx) {
                Poll::Ready(true) => continue,
                Poll::Ready(false) => {}
                Poll::Pending => {
                    is_pending = true;
                }
            }
        }

        Poll::Pending
    }
}
