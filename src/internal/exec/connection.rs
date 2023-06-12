use futures::{ready, Stream};
use httpz::ws::{Message, Websocket};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use streamunordered::{StreamUnordered, StreamYield};

use crate::{
    integrations::httpz::PlzNameThisEnum,
    internal::{exec, PinnedOption, PinnedOptionProj},
};

use super::{
    AsyncRuntime, Executor, GenericSubscriptionManager, Request, Response, SubscriptionMap,
};

// TODO: Seal this shit up tight

/// TODO
#[pin_project(project = ConnectionProj)]
pub struct Connection<R, TCtx>
where
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
{
    ctx: TCtx,
    executor: Executor<TCtx, R>,
    map: SubscriptionMap<R>,
    #[pin]
    streams: StreamUnordered<PlzNameThisEnum<TCtx>>,
}

impl<R, TCtx> Connection<R, TCtx>
where
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
{
    pub fn new(ctx: TCtx, executor: Executor<TCtx, R>) -> Self {
        Self {
            ctx,
            executor,
            map: SubscriptionMap::<R>::new(),
            streams: StreamUnordered::new(),
        }
    }

    pub fn exec(&mut self, reqs: Vec<Request>) -> Vec<Response> {
        let mut manager = Some(GenericSubscriptionManager {
            map: &mut self.map,
            queued: None,
        });

        let resps = self
            .executor
            .execute_batch(&self.ctx, reqs, &mut manager, |fut| {
                self.streams
                    .insert(PlzNameThisEnum::ExecRequestFut(PinnedOption::Some(fut)));
            });

        if let Some(queued) = manager.expect("rspc unreachable").queued {
            for s in queued {
                self.streams.insert(PlzNameThisEnum::OwnedStream(s));
            }
        }

        resps
    }
}

impl<R, TCtx> Stream for Connection<R, TCtx>
where
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
{
    // `Option::None` means nothing to report, continue on with poll impl.
    // This could *technically* be the `Option` forced by `Stream` but that would go against the semantic meaning of it.
    type Item = Option<exec::Response>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        // TODO: Terminate when asked to by subscription manager

        match ready!(this.streams.as_mut().poll_next(cx)) {
            Some((a, b)) => match a {
                StreamYield::Item(resp) => Poll::Ready(Some(Some(resp))),
                StreamYield::Finished(f) => {
                    f.remove(this.streams.as_mut());

                    // TODO: Let the frontend know the stream was dropped

                    Poll::Ready(Some(None))
                }
            },
            // If no streams, fall asleep until a new subscription is queued
            None => return Poll::Pending,
        }
    }
}

// TODO: Break file?

// Time to wait for more messages before sending them over the websocket connection.
const BATCH_TIMEOUT: Duration = Duration::from_millis(75);

/// TODO
#[pin_project(project = BatchFutProj)]
pub struct Batcher<R: AsyncRuntime> {
    batch: Vec<exec::Response>,
    #[pin]
    batch_timer: PinnedOption<R::SleepUtilFut>,
}

impl<R: AsyncRuntime> Batcher<R> {
    pub fn new() -> Self {
        Self {
            batch: Vec::with_capacity(4),
            batch_timer: PinnedOption::None,
        }
    }

    pub fn insert(self: Pin<&mut Self>, element: exec::Response) {
        let mut this = self.project();
        this.batch.push(element);
        this.batch_timer.set(PinnedOption::Some(R::sleep_util(
            Instant::now() + BATCH_TIMEOUT,
        )));
    }

    pub fn append(self: Pin<&mut Self>, other: &mut Vec<exec::Response>) {
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

impl<R: AsyncRuntime> Stream for Batcher<R> {
    // `Option::None` means nothing to report, continue on with poll impl.
    // This could *technically* be the `Option` forced by `Stream` but that would go against the semantic meaning of it.
    type Item = Option<String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.batch_timer.as_mut().project() {
            PinnedOptionProj::Some(batch_timer) => match batch_timer.poll(cx) {
                Poll::Ready(()) => {
                    let queue = this.batch.drain(0..this.batch.len()).collect::<Vec<_>>();
                    this.batch_timer.as_mut().set(PinnedOption::None);

                    if queue.len() != 0 {
                        // TODO: Error handling
                        Poll::Ready(Some(Some(serde_json::to_string(&queue).unwrap())))
                    } else {
                        Poll::Ready(Some(None))
                    }
                }
                Poll::Pending => Poll::Ready(Some(None)),
            },
            PinnedOptionProj::None => Poll::Ready(Some(None)),
        }
    }
}
