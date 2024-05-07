use std::any::TypeId;
use std::task::Poll;
use std::{fmt, future::Future, marker::PhantomData, pin::Pin};

use super::result::ProcedureResult;

use super::{builder::GG, ProcedureBuilder};

/// TODO
pub struct Procedure<TCtx = ()> {
    pub(super) handler:
        Box<dyn Fn(TCtx, &mut dyn erased_serde::Deserializer<'_>) -> ProcedureResult>,
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx> Procedure<TCtx> {
    /// TODO
    pub fn builder<R, I>() -> ProcedureBuilder<TCtx, GG<R, I>> {
        ProcedureBuilder {
            phantom: PhantomData,
        }
    }

    // TODO: Export types
    // TODO: Run this procedure

    // TODO: Allow running synchronously
    // TODO: What if this is a stream not a future????
    pub async fn exec(&self, ctx: TCtx, input: ()) -> Result<ProcedureExecResult, ()> {
        todo!();
    }
}

// TODO: Maybe rename after renaming `ProcedureResult`
pub enum ProcedureExecResult {
    Future(ProcedureExecResultFuture),
    // Stream(ProcedureExecResultStream),
}

// TODO: Take closure
// TODO: Only allow `ProcedureResult` to be used once
pub struct ProcedureExecResultFuture(ProcedureResult, fn(&mut ProcedureResult) -> ());

impl Future for ProcedureExecResultFuture {
    type Output = TypeId; // TODO

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.0.inner.as_mut().poll(cx) {
            Poll::Ready(Some(())) => Poll::Ready(self.0.type_id()),
            Poll::Ready(None) => todo!(),
            Poll::Pending => Poll::Pending,
        }
    }
}

// TODO: Make this public in `procedure.rs`
// pub struct ProcedureExecResultStream(ProcedureResult);
