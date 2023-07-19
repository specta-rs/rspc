use std::{
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{ready, Sink, Stream};
use pin_project_lite::pin_project;
use serde_json::Value;
use streamunordered::{StreamUnordered, StreamYield};

use super::{
    AsyncRuntime, Executor, IncomingMessage, OwnedStream, Request, Response, StreamOrFut,
    SubscriptionManager, SubscriptionSet,
};
use crate::internal::{
    exec::{self, ResponseInner},
    PinnedOption, PinnedOptionProj,
};

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

struct ConnectionSubscriptionManager<'a, TCtx> {
    pub map: &'a mut SubscriptionSet,
    pub to_abort: Option<Vec<u32>>,
    pub queued: Option<Vec<OwnedStream<TCtx>>>,
}

impl<'a, TCtx: Clone + Send + 'static> SubscriptionManager<TCtx>
    for ConnectionSubscriptionManager<'a, TCtx>
{
    type Set<'m> = &'m mut SubscriptionSet where Self: 'm;

    fn queue(&mut self, stream: OwnedStream<TCtx>) {
        match &mut self.queued {
            Some(queued) => {
                queued.push(stream);
            }
            None => self.queued = Some(vec![stream]),
        }
    }

    fn subscriptions(&mut self) -> Self::Set<'_> {
        self.map
    }

    fn abort_subscription(&mut self, id: u32) {
        self.to_abort.get_or_insert_with(Vec::new).push(id);
    }
}

pin_project! {
    #[project = BatchFutProj]
    struct Batcher<R: AsyncRuntime> {
        batch: Vec<exec::Response>,
        #[pin]
        batch_timer: PinnedOption<R::SleepUtilFut>,
    }
}

impl<R: AsyncRuntime> Batcher<R> {
    fn insert(self: Pin<&mut Self>, element: exec::Response) {
        let mut this = self.project();
        this.batch.push(element);
        this.batch_timer
            .set(R::sleep_util(Instant::now() + BATCH_TIMEOUT).into());
    }

    fn append(self: Pin<&mut Self>, other: &mut Vec<exec::Response>) {
        if other.is_empty() {
            return;
        }

        let mut this = self.project();
        this.batch.append(other);
        this.batch_timer
            .set(R::sleep_util(Instant::now() + BATCH_TIMEOUT).into());
    }
}

pin_project! {
    #[project = ConnectionProj]
    struct Connection<TCtx> {
        ctx: TCtx,
        executor: Executor<TCtx>,
        map: SubscriptionSet,
        #[pin]
        streams: StreamUnordered<StreamOrFut<TCtx>>,

        // TODO: Remove these cause disgusting messes
        steam_to_sub_id: HashMap<usize, u32>,
        sub_id_to_stream: HashMap<u32, usize>,
    }
}

impl<TCtx> Connection<TCtx>
where
    TCtx: Clone + Send + 'static,
{
    pub fn exec(&mut self, reqs: Vec<Request>) -> Vec<Response> {
        let mut manager = Some(ConnectionSubscriptionManager {
            map: &mut self.map,
            to_abort: None,
            queued: None,
        });

        let resps = self
            .executor
            .execute_batch(&self.ctx, reqs, &mut manager, |fut| {
                let fut_id = fut.id;
                let token = self.streams.insert(StreamOrFut::ExecRequestFut { fut });
                self.steam_to_sub_id.insert(token, fut_id);
                self.sub_id_to_stream.insert(fut_id, token);
            });

        let manager = manager.expect("rspc unreachable");
        if let Some(to_abort) = manager.to_abort {
            for sub_id in to_abort {
                if let Some(id) = self.sub_id_to_stream.remove(&sub_id) {
                    Pin::new(&mut self.streams).remove(id);
                    self.steam_to_sub_id.remove(&id);
                }
                manager.map.remove(&sub_id);
            }
        }

        if let Some(queued) = manager.queued {
            for stream in queued {
                let sub_id = stream.id;
                let token = self.streams.insert(StreamOrFut::OwnedStream { stream });
                self.steam_to_sub_id.insert(token, sub_id);
                self.sub_id_to_stream.insert(sub_id, token);
            }
        }

        resps
    }
}

type ClearSubscriptionsRx = Option<Box<dyn FnMut(&mut Context<'_>) -> Poll<Option<()>> + Send>>;

pin_project! {
    #[project = ConnectionTaskProj]
    /// An abstraction around a single "connection" which can execute rspc subscriptions.
    ///
    /// For Tauri a "connection" would be for each webpage and for HTTP that is a whole websocket connection.
    ///
    /// This future is spawned onto a thread and coordinates everything. It handles:
    /// - Sending to connection
    /// - Reading from connection
    /// - Executing requests and subscriptions
    /// - Batching responses
    ///
    pub(crate) struct ConnectionTask<R: AsyncRuntime, TCtx, S, E> {
        #[pin]
        conn: Connection<TCtx>,
        #[pin]
        batch: Batcher<R>,

        // Socket
        #[pin]
        socket: S,
        tx_queue: Option<String>,

        // External signal which when called will clear all active subscriptions.
        // This is used by Tauri on window change as the "connection" never shuts down like a websocket would on page reload.
        clear_subscriptions_rx: ClearSubscriptionsRx,

        phantom: PhantomData<E>
    }
}

impl<
        R: AsyncRuntime,
        TCtx: Clone + Send + 'static,
        S: Sink<String, Error = E> + Stream<Item = Result<IncomingMessage, E>> + Send + Unpin,
        E: std::fmt::Debug + std::error::Error,
    > ConnectionTask<R, TCtx, S, E>
{
    #[allow(dead_code)]
    pub fn new(
        ctx: TCtx,
        executor: Executor<TCtx>,
        socket: S,
        clear_subscriptions_rx: ClearSubscriptionsRx,
    ) -> Self {
        Self {
            conn: Connection {
                ctx,
                executor,
                map: SubscriptionSet::new(),
                streams: StreamUnordered::new(),
                steam_to_sub_id: HashMap::new(),
                sub_id_to_stream: HashMap::new(),
            },
            batch: Batcher {
                batch: Vec::with_capacity(4),
                batch_timer: PinnedOption::None,
            },
            socket,
            tx_queue: None,
            clear_subscriptions_rx,
            phantom: PhantomData,
        }
    }

    /// Poll sending
    fn poll_send(this: &mut ConnectionTaskProj<R, TCtx, S, E>, cx: &mut Context<'_>) -> Poll<()> {
        // If nothing in `tx_queue`, poll the batcher to populate it
        if this.tx_queue.is_none() {
            let mut batch = this.batch.as_mut().project();
            if let PinnedOptionProj::Some { v: batch_timer } = batch.batch_timer.as_mut().project()
            {
                ready!(batch_timer.poll(cx));

                let queue = batch.batch.drain(0..batch.batch.len()).collect::<Vec<_>>();
                batch.batch_timer.as_mut().set(PinnedOption::None);

                if !queue.is_empty() {
                    match serde_json::to_string(&queue) {
                        Ok(s) => *this.tx_queue = Some(s),
                        // This error isn't really handled and that is because if `queue` which is a `Vec<Response>` fails serialization, well we are gonna wanna send a `Response` with the error which will also most likely fail serialization.
                        // It's important to note the user provided types are converted to `serde_json::Value` prior to being put into this type so this will only ever fail on internal types.
                        Err(err) => {
                            #[allow(clippy::panic)]
                            {
                                #[cfg(debug_assertions)]
                                panic!("rspc internal serialization error: {}", err);
                            }
                        }
                    }
                }
            }
        }

        // If something is queued to send
        if this.tx_queue.is_some() {
            // Wait until the socket is ready for sending
            if let Err(_err) = ready!(this.socket.as_mut().poll_ready(cx)) {
                #[cfg(feature = "tracing")]
                tracing::error!("Error waiting for websocket to be ready: {}", _err);

                return ().into();
            };

            let item = this
                .tx_queue
                .take()
                // We check it is `Some(_)` every poll but defer taking it from the `Option` until the socket is ready
                .expect("rspc unreachable");

            if let Err(_err) = this.socket.as_mut().start_send(item) {
                #[cfg(feature = "tracing")]
                tracing::error!("Error sending message to websocket: {}", _err);
            }
        }

        // Flush the previously sent data if any is pending
        if let Err(_err) = ready!(this.socket.as_mut().poll_flush(cx)) {
            #[cfg(feature = "tracing")]
            tracing::error!("Error flushing message to websocket: {}", _err);
        }

        ().into()
    }

    /// Poll receiving
    fn poll_recv(
        this: &mut ConnectionTaskProj<R, TCtx, S, E>,
        cx: &mut Context<'_>,
    ) -> Poll<PollResult> {
        match ready!(this.socket.as_mut().poll_next(cx)) {
            Some(Ok(msg)) => {
                let res = match msg {
                    IncomingMessage::Msg(json) => json,
                    IncomingMessage::Close => return PollResult::Complete.into(),
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

                        // TODO: Send report of error to frontend but who do we correlated them????
                    }
                }

                PollResult::QueueSend
            }
            Some(Err(_err)) => {
                #[cfg(feature = "tracing")]
                tracing::debug!("Error reading from websocket connection: {:?}", _err);

                // TODO: Send report of error to frontend but who do we correlated them????

                PollResult::QueueSend
            }
            None => PollResult::Complete,
        }
        .into()
    }

    /// Poll active streams
    fn poll_streams(
        this: &mut ConnectionTaskProj<R, TCtx, S, E>,
        cx: &mut Context<'_>,
    ) -> Poll<PollResult> {
        let mut conn = this.conn.as_mut().project();
        for _ in 0..conn.streams.len() {
            match ready!(conn.streams.as_mut().poll_next(cx)) {
                Some((a, _)) => match a {
                    StreamYield::Item(resp) => {
                        this.batch.as_mut().insert(resp);
                        return PollResult::QueueSend.into();
                    }
                    StreamYield::Finished(f) => {
                        f.take(conn.streams.as_mut());
                    }
                },
                // If no streams, fall asleep until a new subscription is queued
                None => {}
            }
        }

        PollResult::Progressed.into()
    }

    fn complete(this: &mut ConnectionTaskProj<R, TCtx, S, E>) {
        #[cfg(feature = "tracing")]
        tracing::trace!("Shutting down websocket connection");

        Self::shutdown_all_streams(this);
    }

    fn shutdown_all_streams(this: &mut ConnectionTaskProj<R, TCtx, S, E>) {
        let mut conn = this.conn.as_mut().project();

        // TODO: This can be improved by: https://github.com/jonhoo/streamunordered/pull/5
        for (token, _) in conn.steam_to_sub_id.drain() {
            conn.streams.as_mut().remove(token);
        }
        conn.steam_to_sub_id.drain().for_each(drop);
        conn.map.drain().for_each(drop);
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
        let mut is_done = false;
        let mut should_send = false;

        while !is_pending || should_send {
            should_send = false;

            if let Some(recv) = &mut this.clear_subscriptions_rx {
                match (recv)(cx) {
                    Poll::Ready(Some(())) => Self::shutdown_all_streams(&mut this),
                    Poll::Ready(None) => *this.clear_subscriptions_rx = None,
                    Poll::Pending => is_pending = true,
                }
            }

            if Self::poll_send(&mut this, cx).is_pending() {
                is_pending = true;
            }

            if is_done {
                if is_pending {
                    return Poll::Pending;
                }

                #[cfg(debug_assertions)]
                if !this.batch.batch.is_empty() {
                    unreachable!("`ConnectionTask::poll_send` is complete but did not send all queued messages");
                }

                return Poll::Ready(());
            }

            match Self::poll_recv(&mut this, cx) {
                Poll::Ready(PollResult::Complete) => {
                    is_done = true;
                    Self::complete(&mut this);
                    continue;
                }
                Poll::Ready(PollResult::Progressed) => {}
                Poll::Ready(PollResult::QueueSend) => {
                    should_send = true;
                    continue;
                }
                Poll::Pending => {
                    is_pending = true;
                }
            }

            match Self::poll_streams(&mut this, cx) {
                Poll::Ready(PollResult::Complete) => {
                    #[cfg(debug_assertions)]
                    unreachable!(
                        "`ConnectionTask::poll_streams` attempted to complete the connection!"
                    );

                    #[cfg(not(debug_assertions))]
                    continue;
                }
                Poll::Ready(PollResult::Progressed) => {}
                Poll::Ready(PollResult::QueueSend) => {
                    should_send = true;
                    continue;
                }
                Poll::Pending => {
                    is_pending = true;
                }
            }
        }

        Poll::Pending
    }
}
