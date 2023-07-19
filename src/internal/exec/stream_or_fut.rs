use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use pin_project_lite::pin_project;

use crate::internal::{exec, PinnedOption, PinnedOptionProj};

use super::{ExecRequestFut, OwnedStream};

mod private {
    use super::*;

    pin_project! {
        /// TODO
        #[project = StreamOrFutProj]
        pub enum StreamOrFut<TCtx> {
            OwnedStream {
                #[pin]
                stream: OwnedStream<TCtx>
            },
            ExecRequestFut {
                #[pin]
                fut: ExecRequestFut,
            },
            // When the underlying stream shutdowns we yield a shutdown message. Once it is yielded we need to yield a `None` to tell the poller we are done.
            PendingDone,
            Done,
        }
    }

    impl<TCtx: 'static> Stream for StreamOrFut<TCtx> {
        type Item = exec::Response;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.as_mut().project() {
                StreamOrFutProj::OwnedStream { mut stream } => {
                    Poll::Ready(Some(match ready!(stream.as_mut().poll_next(cx)) {
                        Some(r) => exec::Response {
                            id: stream.id,
                            inner: match r {
                                Ok(v) => exec::ResponseInner::Value(v),
                                Err(err) => exec::ResponseInner::Error(err.into()),
                            },
                        },
                        None => {
                            let id = stream.id;
                            cx.waker().wake_by_ref(); // No wakers set so we set one
                            self.set(StreamOrFut::PendingDone);
                            exec::Response {
                                id,
                                inner: exec::ResponseInner::Complete,
                            }
                        }
                    }))
                }
                StreamOrFutProj::ExecRequestFut { fut } => fut.poll(cx).map(|v| {
                    cx.waker().wake_by_ref(); // No wakers set so we set one
                    self.set(StreamOrFut::PendingDone);
                    Some(v)
                }),
                StreamOrFutProj::PendingDone => {
                    self.set(StreamOrFut::Done);
                    Poll::Ready(None)
                }
                StreamOrFutProj::Done => {
                    #[cfg(debug_assertions)]
                    panic!("`StreamOrFut` polled after completion");

                    #[cfg(not(debug_assertions))]
                    Poll::Ready(None)
                }
            }
        }
    }
}

#[cfg(feature = "unstable")]
pub use private::StreamOrFut;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::StreamOrFut;
