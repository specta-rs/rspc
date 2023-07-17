use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use pin_project::pin_project;

use crate::internal::{exec, PinnedOption, PinnedOptionProj};

use super::{ExecRequestFut, OwnedStream};

/// TODO
#[pin_project(project = StreamOrFutProj)]
pub(crate) enum StreamOrFut<TCtx: 'static> {
    OwnedStream(#[pin] OwnedStream<TCtx>),
    ExecRequestFut(#[pin] ExecRequestFut),
    Done(u32),
}

impl<TCtx: 'static> StreamOrFut<TCtx> {
    pub fn id(&self) -> u32 {
        match self {
            StreamOrFut::OwnedStream(v) => v.id,
            StreamOrFut::ExecRequestFut(v) => v.id,
            StreamOrFut::Done(id) => *id,
        }
    }
}

impl<TCtx: 'static> Stream for StreamOrFut<TCtx> {
    type Item = exec::Response;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().project() {
            StreamOrFutProj::OwnedStream(s) => {
                let s = s.project();

                Poll::Ready(ready!(s.reference.poll_next(cx)).map(|r| exec::Response {
                    id: *s.id,
                    inner: match r {
                        Ok(v) => exec::ResponseInner::Value(v),
                        Err(err) => exec::ResponseInner::Error(err.into()),
                    },
                }))
            }
            StreamOrFutProj::ExecRequestFut(s) => s.poll(cx).map(|v| {
                self.set(StreamOrFut::Done(v.id));
                Some(v)
            }),
            StreamOrFutProj::Done(_) => Poll::Ready(None),
        }
    }
}
