use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use pin_project_lite::pin_project;
use serde_json::Value;

use crate::{
    internal::{
        exec::{self, Response, ResponseInner},
        middleware::{RequestContext, STATE},
        PinnedOption, PinnedOptionProj, ProcedureStore,
    },
    BuiltRouter, ExecError,
};

use super::{ExecutorResult, RspcStream};

/// TODO
pub struct RequestFuture {
    id: u32,

    // You will notice this is a `Stream` not a `Future` like would be implied by the struct.
    // rspc's whole middleware system only works on `Stream`'s cause it makes life easier so we change to & from a `Future` at the start/end.
    stream: Pin<Box<dyn RspcStream<Item = Result<Value, ExecError>> + Send>>,
}

impl RequestFuture {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn exec<TCtx: 'static>(
        ctx: TCtx,
        procedures: *const ProcedureStore<TCtx>,
        req: RequestContext,
        input: Option<Value>,
    ) -> ExecutorResult {
        // TODO: This unsafe is not coupled to the Arc which is bad
        match unsafe { &*procedures }.store.get(req.path.as_ref()) {
            Some(procedure) => ExecutorResult::FutureResponse(Self {
                id: req.id,
                stream: procedure
                    .exec
                    .dyn_call(ctx, input.unwrap_or(Value::Null), req),
            }),
            None => ExecutorResult::Response(Response {
                id: req.id,
                inner: ResponseInner::Error(ExecError::OperationNotFound.into()),
            }),
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Response> {
        match self.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(result))) => Poll::Ready(Response {
                id: self.id,
                inner: ResponseInner::Value(result),
            }),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Response {
                id: self.id,
                inner: ResponseInner::Error(err.into()),
            }),
            Poll::Ready(None) => Poll::Ready(Response {
                id: self.id,
                inner: ResponseInner::Error(ExecError::ErrStreamEmpty.into()),
            }),
            Poll::Pending => {
                // TODO: Do this body stuff for `OwnedStream` too by putting it in `StreamOrFut`
                // TODO: Move `STATE` data in local state incase future is moved onto thread.

                // let wants_body = STATE.with(|w| w.borrow_mut().waker.is_some());
                // println!("{:?}", wants_body);
                // if wants_body {
                //     STATE.with(|w| {
                //         let mut w = w.borrow_mut();
                //         w.chunk = Some(Bytes::from("Hello World"));
                //         w.waker.take().expect("unreachable").wake(); // TODO: Use waker vs just looping?
                //     });
                // }

                Poll::Pending
            }
        }
    }
}

impl Future for RequestFuture {
    type Output = Response;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Self::poll(&mut self, cx)
    }
}

/// TODO
pub struct RspcTask<TCtx>(Inner<TCtx>);

impl<TCtx> From<RequestFuture> for RspcTask<TCtx> {
    fn from(value: RequestFuture) -> Self {
        Self(Inner::Future(value))
    }
}

impl<TCtx: 'static> RspcTask<TCtx> {
    // TODO: Break this out
    pub(crate) fn new_stream(
        router: Arc<BuiltRouter<TCtx>>,
        ctx: TCtx,
        input: Option<Value>,
        req: RequestContext,
    ) -> Result<Self, u32> {
        let stream: *const _ = match router.subscriptions.store.get(req.path.as_ref()) {
            Some(v) => v,
            None => return Err(req.id),
        };

        let id = req.id;

        // SAFETY: Trust me bro
        let stream = unsafe { &*stream }
            .exec
            .dyn_call(ctx, input.unwrap_or(Value::Null), req);

        Ok(Self(Inner::Stream {
            _arc: router,
            reference: stream,
            id,
        }))
    }

    pub fn id(&self) -> u32 {
        match self.0 {
            Inner::Stream { id, .. } => id,
            Inner::Future(ref fut) => fut.id,
            Inner::PendingDone { id } => id,
            Inner::Done { id } => id,
        }
    }
}

enum Inner<TCtx> {
    Stream {
        id: u32,
        // We MUST hold the `Arc` so it doesn't get dropped while the stream exists from it.
        _arc: Arc<BuiltRouter<TCtx>>,
        // The stream to poll
        reference: Pin<Box<dyn RspcStream<Item = Result<Value, ExecError>> + Send>>,
    },
    Future(RequestFuture),
    // When the underlying stream yields `None` we map it to a "complete" message and change to this state.
    // This state will yield a `None` to tell the poller we are actually done.
    PendingDone {
        id: u32,
    },
    Done {
        id: u32,
    },
}

impl<TCtx: 'static> Stream for RspcTask<TCtx> {
    type Item = exec::Response;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.0 {
            Inner::Stream {
                id,
                ref mut reference,
                ..
            } => {
                Poll::Ready(Some(match ready!(reference.as_mut().poll_next(cx)) {
                    Some(r) => exec::Response {
                        id: *id,
                        inner: match r {
                            Ok(v) => exec::ResponseInner::Value(v),
                            Err(err) => exec::ResponseInner::Error(err.into()),
                        },
                    },
                    None => {
                        let id = *id;
                        cx.waker().wake_by_ref(); // No wakers set so we set one
                        self.set(Self(Inner::PendingDone { id }));
                        exec::Response {
                            id,
                            inner: exec::ResponseInner::Complete,
                        }
                    }
                }))
            }
            Inner::Future(fut) => {
                let id = fut.id;
                fut.poll(cx).map(|v| {
                    cx.waker().wake_by_ref(); // No wakers set so we set one
                    self.set(Self(Inner::PendingDone { id }));
                    Some(v)
                })
            }
            Inner::PendingDone { id } => {
                let id = *id;
                self.set(Self(Inner::Done { id }));
                Poll::Ready(None)
            }
            Inner::Done { .. } => {
                #[cfg(debug_assertions)]
                panic!("`StreamOrFut` polled after completion");

                #[cfg(not(debug_assertions))]
                Poll::Ready(None)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            Inner::Stream { ref reference, .. } => reference.size_hint(),
            Inner::Future { .. } => (0, Some(1)),
            Inner::PendingDone { .. } => (0, Some(0)),
            Inner::Done { .. } => (0, Some(0)),
        }
    }
}
