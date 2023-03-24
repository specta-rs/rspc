use std::future::{ready, Future, Ready};

use serde_json::Value;

use crate::internal::RequestContext;

use super::{Fut, Ret};

// TODO: Probs split these functions into two traits???
pub trait Executable<TLCtx, TState, TRet>: Send + 'static {
    type Fut: Future<Output = TRet>;

    fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut;

    // This function exists so that `Self::Fut` doesn't need to deal with lifetimes or cloning the next middleware
    // TODO: Remove default implementation
    fn call2(&self, result: TRet) -> () {
        todo!();
    }
}

impl<TLCtx, TState, TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + Send + 'static>
    Executable<TLCtx, TState, TRet> for TFunc
{
    type Fut = TFut;

    fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut {
        (self)()
    }
}
