use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use pin_project_lite::pin_project;

use crate::internal::{exec, PinnedOption, PinnedOptionProj};

use super::ExecRequestFut;

mod private {
    use std::sync::Arc;

    use serde_json::Value;

    use crate::{internal::middleware::RequestContext, BuiltRouter, ExecError};

    use super::*;

    pin_project! {
        /// TODO
        #[project = StreamOrFutProj]
        pub enum StreamOrFut<TCtx> {
            Stream {
                id: u32,
                // We MUST hold the `Arc` so it doesn't get dropped while the stream exists from it.
                arc: Arc<BuiltRouter<TCtx>>,
                // The stream to poll
                #[pin]
                reference: Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>,
            },
            Future {
                #[pin]
                fut: ExecRequestFut,
            },
            // When the underlying stream shutdowns we yield a shutdown message. Once it is yielded we need to yield a `None` to tell the poller we are done.
            PendingDone {
                id: u32
            },
            Done { id: u32 },
        }
    }

    impl<TCtx: 'static> StreamOrFut<TCtx> {
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

            Ok(Self::Stream {
                arc: router,
                reference: stream,
                id,
            })
        }

        pub fn id(&self) -> u32 {
            match self {
                StreamOrFut::Stream { id, .. } => *id,
                StreamOrFut::Future { fut } => fut.id,
                StreamOrFut::PendingDone { id } => *id,
                StreamOrFut::Done { id } => *id,
            }
        }
    }

    impl<TCtx: 'static> Stream for StreamOrFut<TCtx> {
        type Item = exec::Response;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.as_mut().project() {
                StreamOrFutProj::Stream {
                    id, mut reference, ..
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
                            self.set(StreamOrFut::PendingDone { id });
                            exec::Response {
                                id,
                                inner: exec::ResponseInner::Complete,
                            }
                        }
                    }))
                }
                StreamOrFutProj::Future { fut } => {
                    let id = fut.id;
                    fut.poll(cx).map(|v| {
                        cx.waker().wake_by_ref(); // No wakers set so we set one
                        self.set(StreamOrFut::PendingDone { id });
                        Some(v)
                    })
                }
                StreamOrFutProj::PendingDone { id } => {
                    let id = *id;
                    self.set(StreamOrFut::Done { id });
                    Poll::Ready(None)
                }
                StreamOrFutProj::Done { .. } => {
                    #[cfg(debug_assertions)]
                    panic!("`StreamOrFut` polled after completion");

                    #[cfg(not(debug_assertions))]
                    Poll::Ready(None)
                }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                StreamOrFut::Stream { reference, .. } => reference.size_hint(),
                StreamOrFut::Future { .. } => (0, Some(1)),
                StreamOrFut::PendingDone { .. } => (0, Some(0)),
                StreamOrFut::Done { .. } => (0, Some(0)),
            }
        }
    }
}

#[cfg(feature = "unstable")]
pub use private::StreamOrFut;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::StreamOrFut;
