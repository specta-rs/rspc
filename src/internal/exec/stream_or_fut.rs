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
            Done,
        }
    }

    impl<TCtx: 'static> Stream for StreamOrFut<TCtx> {
        type Item = exec::Response;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.as_mut().project() {
                StreamOrFutProj::OwnedStream { stream } => {
                    let s = stream.project();

                    Poll::Ready(Some(match ready!(s.reference.poll_next(cx)) {
                        Some(r) => exec::Response {
                            id: *s.id,
                            inner: match r {
                                Ok(v) => exec::ResponseInner::Value(v),
                                Err(err) => exec::ResponseInner::Error(err.into()),
                            },
                        },
                        None => {
                            let id = *s.id;
                            self.set(StreamOrFut::Done);
                            exec::Response {
                                id,
                                inner: exec::ResponseInner::Complete,
                            }
                        }
                    }))
                }
                StreamOrFutProj::ExecRequestFut { fut } => fut.poll(cx).map(|v| {
                    self.set(StreamOrFut::Done);
                    Some(v)
                }),
                StreamOrFutProj::Done { .. } => Poll::Ready(None),
            }
        }
    }
}

#[cfg(feature = "unstable")]
pub use private::StreamOrFut;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::StreamOrFut;
