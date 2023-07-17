use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{ready, Sink, Stream};
use pin_project::pin_project;
use serde_json::Value;

use super::{AsyncRuntime, Connection, IncomingMessage};
use crate::internal::{exec, PinnedOption, PinnedOptionProj};

// Time to wait for more messages before sending them over the websocket connection.
const BATCH_TIMEOUT: Duration = Duration::from_millis(15);

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

/// TODO
#[pin_project(project = BatchFutProj)]
struct Batcher<R: AsyncRuntime> {
    batch: Vec<exec::Response>,
    #[pin]
    batch_timer: PinnedOption<R::SleepUtilFut>,
}

impl<R: AsyncRuntime> Batcher<R> {
    fn insert(self: Pin<&mut Self>, element: exec::Response) {
        let mut this = self.project();
        this.batch.push(element);
        this.batch_timer.set(PinnedOption::Some(R::sleep_util(
            Instant::now() + BATCH_TIMEOUT,
        )));
    }

    fn append(self: Pin<&mut Self>, other: &mut Vec<exec::Response>) {
        if other.len() == 0 {
            return;
        }

        let mut this = self.project();
        this.batch.append(other);
        this.batch_timer.set(PinnedOption::Some(R::sleep_util(
            Instant::now() + BATCH_TIMEOUT,
        )));
    }
}

/// TODO
#[pin_project(project = ConnectionTaskProj)]
pub(crate) struct ConnectionTask<
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
    S: Sink<String, Error = E> + Stream<Item = Result<IncomingMessage, E>> + Send + Unpin,
    E: std::fmt::Debug + std::error::Error,
> {
    #[pin]
    conn: Connection<R, TCtx>,
    #[pin]
    batch: Batcher<R>,

    // Socket
    #[pin]
    socket: S,
    tx_queue: Option<String>,
}

impl<
        R: AsyncRuntime,
        TCtx: Clone + Send + 'static,
        S: Sink<String, Error = E> + Stream<Item = Result<IncomingMessage, E>> + Send + Unpin,
        E: std::fmt::Debug + std::error::Error,
    > ConnectionTask<R, TCtx, S, E>
{
    pub fn new(conn: Connection<R, TCtx>, socket: S) -> Self {
        Self {
            conn,
            batch: Batcher {
                batch: Vec::with_capacity(4),
                batch_timer: PinnedOption::None,
            },
            socket,
            tx_queue: None,
        }
    }

    /// Poll sending
    fn poll_send(
        this: &mut ConnectionTaskProj<R, TCtx, S, E>,
        cx: &mut Context<'_>,
    ) -> Poll<PollResult> {
        // If nothing in `tx_queue`, poll the batcher to populate it
        if this.tx_queue.is_none() {
            let mut batch = this.batch.as_mut().project();
            if let PinnedOptionProj::Some(batch_timer) = batch.batch_timer.as_mut().project() {
                ready!(batch_timer.poll(cx));

                let queue = batch.batch.drain(0..batch.batch.len()).collect::<Vec<_>>();
                batch.batch_timer.as_mut().set(PinnedOption::None);

                if queue.len() != 0 {
                    // TODO: Error handling
                    *this.tx_queue = Some(serde_json::to_string(&queue).unwrap());
                }
            }
        }

        // If something is queued to send
        if let Some(_) = this.tx_queue {
            // Wait until the socket is ready for sending
            if let Err(err) = ready!(this.socket.as_mut().poll_ready(cx)) {
                #[cfg(feature = "tracing")]
                tracing::error!("Error waiting for websocket to be ready: {}", err);

                return PollResult::Progressed.into();
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

        PollResult::Progressed.into()
    }

    /// Poll receiving
    fn poll_recv(
        this: &mut ConnectionTaskProj<R, TCtx, S, E>,
        cx: &mut Context<'_>,
    ) -> Poll<PollResult> {
        match ready!(this.socket.as_mut().poll_next(cx)) {
            Some(Ok(msg)) => {
                let res = match msg.into() {
                    IncomingMessage::Msg(json) => json,
                    IncomingMessage::Close => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Shutting down websocket connection");

                        // TODO: Terminate all subscriptions
                        // TODO: Tell frontend all subscriptions were terminated

                        return PollResult::Complete.into();
                    }
                    IncomingMessage::Skip => return PollResult::Progressed.into(),
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
                        println!("D {_err:?}");
                    }
                }

                PollResult::QueueSend
            }
            Some(Err(_err)) => {
                #[cfg(feature = "tracing")]
                tracing::debug!("Error reading from websocket connection: {:?}", _err);

                // TODO: Send report of error to frontend
                println!("E {_err:?}");

                PollResult::QueueSend
            }
            None => {
                #[cfg(feature = "tracing")]
                tracing::debug!("Shutting down websocket connection");

                // TODO: Terminate all subscriptions
                // TODO: Tell frontend all subscriptions were terminated

                PollResult::Complete
            }
        }
        .into()
    }

    /// Poll active streams
    fn poll_streams(
        this: &mut ConnectionTaskProj<R, TCtx, S, E>,
        cx: &mut Context<'_>,
    ) -> Poll<PollResult> {
        if let Some(batch) = ready!(this.conn.as_mut().poll_next(cx)).expect("rspc unreachable") {
            this.batch.as_mut().insert(batch);
            return PollResult::QueueSend.into();
        }

        PollResult::Progressed.into()
    }
}

impl<
        R: AsyncRuntime,
        TCtx: Clone + Send + 'static,
        S: Sink<String, Error = E> + Stream<Item = Result<IncomingMessage, E>> + Send + Unpin,
        E: std::fmt::Debug + std::error::Error,
    > Future for ConnectionTask<R, TCtx, S, E>
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
                Poll::Ready(PollResult::Complete) => return Poll::Ready(()),
                Poll::Ready(PollResult::Progressed) => {}
                Poll::Ready(PollResult::QueueSend) => continue,
                Poll::Pending => {
                    is_pending = true;
                }
            }

            match Self::poll_streams(&mut this, cx) {
                Poll::Ready(PollResult::Complete) => return Poll::Ready(()),
                Poll::Ready(PollResult::Progressed) => {}
                Poll::Ready(PollResult::QueueSend) => continue,
                Poll::Pending => {
                    is_pending = true;
                }
            }
        }

        Poll::Pending
    }
}
