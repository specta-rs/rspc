use futures::{ready, Stream};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use streamunordered::{StreamUnordered, StreamYield};

use crate::internal::{exec, PinnedOption, PinnedOptionProj};

use super::{
    AsyncRuntime, ExecRequestFut, Executor, GenericSubscriptionManager, OwnedStream, Request,
    Response, SubscriptionMap,
};

// TODO: Seal this shit up tight

/// TODO
#[pin_project(project = StreamOrFutProj)]
enum StreamOrFut<TCtx: 'static> {
    OwnedStream(#[pin] OwnedStream<TCtx>),
    ExecRequestFut(#[pin] PinnedOption<ExecRequestFut>),
}

impl<TCtx: 'static> Stream for StreamOrFut<TCtx> {
    type Item = exec::Response;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            StreamOrFutProj::OwnedStream(s) => {
                let s = s.project();

                let v = ready!(s.reference.poll_next(cx));

                Poll::Ready(v.map(|r| exec::Response {
                    id: *s.id,
                    result: match r {
                        Ok(v) => exec::ValueOrError::Value(v),
                        Err(err) => exec::ValueOrError::Error(err.into()),
                    },
                }))
            }
            StreamOrFutProj::ExecRequestFut(mut s) => match s.as_mut().project() {
                PinnedOptionProj::Some(ss) => ss.poll(cx).map(|v| {
                    s.set(PinnedOption::None);
                    Some(v)
                }),
                PinnedOptionProj::None => Poll::Ready(None),
            },
        }
    }
}

/// TODO
#[pin_project(project = ConnectionProj)]
pub(crate) struct Connection<R, TCtx>
where
    R: AsyncRuntime,
    TCtx: Clone + Send + 'static,
{
    ctx: TCtx,
    executor: Executor<TCtx, R>,
    map: SubscriptionMap<R>,
    #[pin]
    streams: StreamUnordered<StreamOrFut<TCtx>>,
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
                    .insert(StreamOrFut::ExecRequestFut(PinnedOption::Some(fut)));
            });

        if let Some(queued) = manager.expect("rspc unreachable").queued {
            for s in queued {
                self.streams.insert(StreamOrFut::OwnedStream(s));
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

    // WARNING: The caller must call this in a loop until they receive a `Poll::Pending` event
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        // TODO: Terminate when asked to by subscription manager

        match ready!(this.streams.as_mut().poll_next(cx)) {
            Some((a, _)) => match a {
                StreamYield::Item(resp) => Poll::Ready(Some(Some(resp))),
                StreamYield::Finished(f) => {
                    f.remove(this.streams.as_mut());

                    // TODO: Let the frontend know the stream was dropped

                    Poll::Ready(Some(None))
                }
            },
            // If no streams, fall asleep until a new subscription is queued
            None => Poll::Pending,
        }
    }
}
