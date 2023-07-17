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
