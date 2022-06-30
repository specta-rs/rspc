use std::{
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use serde_json::Value;

use crate::{ConcreteArg, ExecError};

/// TODO
pub(crate) type MiddlewareChainBase<TCtx> =
    Box<dyn Fn(TCtx, ConcreteArg) -> Result<MiddlewareResult, ExecError> + Send + Sync>;

/// TODO
pub(crate) type OperationHandler<TLayerCtx> =
    Box<dyn Fn(TLayerCtx, ConcreteArg) -> Result<MiddlewareResult, ExecError> + Send + Sync>;

/// TODO
pub(crate) type MiddlewareChain<TCtx, TLayerCtx> =
    Box<dyn Fn(OperationHandler<TLayerCtx>) -> MiddlewareChainBase<TCtx> + Send + Sync>;

/// TODO
pub enum MiddlewareResult {
    Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send + Sync>>),
    Sync(Value),
    Gone,
}

impl Future for MiddlewareResult {
    type Output = Result<Value, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this {
            MiddlewareResult::Future(fut) => Pin::new(fut).poll(cx),
            MiddlewareResult::Sync(_) => {
                let v = match mem::replace(this, MiddlewareResult::Gone) {
                    MiddlewareResult::Sync(v) => v,
                    _ => unreachable!(),
                };

                Poll::Ready(Ok(v))
            }
            MiddlewareResult::Gone => unreachable!(),
        }
    }
}
