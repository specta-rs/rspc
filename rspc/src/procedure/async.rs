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
    Value(ProcedureResult),
    Future(Pin<Box<dyn Future<Output = ProcedureResult> + Send>>),
}

pub struct ProcedureExecResultFuture(Inner);

impl Future for ProcedureExecResultFuture {
    type Output = Result<ProcedureResult, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!();
    }
}

pub struct ProcedureExecResultStream; // TODO
