use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use pin_project_lite::pin_project;

use crate::internal::{exec, PinnedOption, PinnedOptionProj};

use super::{ExecRequestFut, OwnedStream};

pin_project! {
    /// TODO
    #[project = StreamOrFutProj]
    pub(crate) enum StreamOrFut<TCtx> {
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

                Poll::Ready(ready!(s.reference.poll_next(cx)).map(|r| exec::Response {
                    id: *s.id,
                    inner: match r {
                        Ok(v) => exec::ResponseInner::Value(v),
                        Err(err) => exec::ResponseInner::Error(err.into()),
                    },
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
