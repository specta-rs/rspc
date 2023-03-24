use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
};

use serde_json::Value;

use crate::internal::RequestContext;

use super::{Fut, Ret};

// TODO: Maybe remove `TMarker` for this now if it's not still being used
pub trait Executable<TLCtx, TState, TRet, TMarker>: Send + 'static {
    type Fut: Future<Output = TRet>;

    fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut;

    // TODO: Probs split this out into another trait if it's needed
    // This function exists so that `Self::Fut` doesn't need to deal with lifetimes or cloning the next middleware
    // TODO: Remove default implementation
    // fn call2(&self, result: TRet) -> () {
    //     todo!();
    // }
}

// TODO: Remove this
pub struct Demo<A, B, C>(pub(crate) PhantomData<(A, B, C)>);
impl<TLCtx, TState, TRet> Executable<TLCtx, TState, TRet, ()> for Demo<TLCtx, TState, TRet>
where
    TLCtx: Send + 'static,
    TState: Send + 'static,
    TRet: Send + 'static,
{
    type Fut = Ready<TRet>;

    fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut {
        todo!()
    }
}
