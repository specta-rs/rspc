use std::{
    fmt,
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
        middleware::RequestContext,
        Body, PinnedOption, PinnedOptionProj,
    },
    BuiltRouter, ExecError,
};

use super::arc_ref::ArcRef;

// TODO: Can we have a public method to convert this into a `RspcTask` by internally grabbing `self.stream` and treating it as a stream???? -> Will we end up with subscriptions like start, done messages being sent?

/// TODO
pub struct RequestFuture {
    pub(crate) id: u32,

    // You will notice this is a `Stream` not a `Future` like would be implied by the struct.
    // rspc's whole middleware system only uses `Stream`'s cause it makes life easier so we change to & from a `Future` at the start/end.
    pub(crate) stream: ArcRef<Pin<Box<dyn Body + Send>>>,
}

impl fmt::Debug for RequestFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestFuture")
            .field("id", &self.id)
            .finish()
    }
}

impl Future for RequestFuture {
    type Output = Response;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Response {
            id: self.id,
            inner: match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(result))) => ResponseInner::Value(result),
                Poll::Ready(Some(Err(err))) => ResponseInner::Error(err.into()),
                Poll::Ready(None) => ResponseInner::Error(ExecError::ErrStreamEmpty.into()),
                Poll::Pending => return Poll::Pending,
            },
        })
    }
}
