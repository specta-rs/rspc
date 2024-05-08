use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::result::ProcedureResult;

// TODO: Reexport all this in `procedure.rs`

pub enum ProcedureExecResult {
    Future(ProcedureExecResultFuture),
    Stream(ProcedureExecResultStream),
}

enum Inner {
    // Value(ProcedureResult), // TODO: Avoid boxing when already ready
    Future(Pin<Box<dyn Future<Output = ProcedureResult> + Send>>),
}

pub struct ProcedureExecResultFuture(Inner);

impl ProcedureExecResultFuture {
    pub(super) fn new<F>(future: F) -> Self
    where
        F: Future<Output = ProcedureResult> + Send + 'static,
    {
        Self(Inner::Future(Box::pin(future)))
    }
}

impl Future for ProcedureExecResultFuture {
    type Output = Result<ProcedureResult, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().0 {
            Inner::Future(fut) => fut.as_mut().poll(cx).map(Ok),
        }
    }
}

pub struct ProcedureExecResultStream; // TODO
