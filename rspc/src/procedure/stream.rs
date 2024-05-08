use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;

use super::output_value::ProcedureResult;

enum Inner {
    // Value(ProcedureResult), // TODO: Avoid boxing when already ready
    Future(Pin<Box<dyn Future<Output = ProcedureResult> + Send>>),
    // Stream(...)
}

pub struct ProcedureStream(Inner);

impl ProcedureStream {
    pub(super) fn new<F>(future: F) -> Self
    where
        F: Future<Output = ProcedureResult> + Send + 'static,
    {
        Self(Inner::Future(Box::pin(future)))
    }
}

impl Stream for ProcedureStream {
    type Item = ProcedureResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.0 {
            Inner::Future(future) => future.as_mut().poll(cx).map(Some), // TODO: Handle emitting `None` when done
        }
    }
}
