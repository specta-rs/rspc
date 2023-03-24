use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
};

use serde_json::Value;

use crate::internal::RequestContext;

use super::{Fut, MwV2, Ret};

// TODO: Probs split these functions into two traits???
pub trait Executable<TLCtx, TState, TRet, TMarker>: Send + 'static {
    type Fut: Future<Output = TRet>;

    fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut;

    // This function exists so that `Self::Fut` doesn't need to deal with lifetimes or cloning the next middleware
    // TODO: Remove default implementation
    fn call2(&self, result: TRet) -> () {
        todo!();
    }
}

// impl<TLCtx, TState, TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + Send + 'static>
//     Executable<TLCtx, TState, TRet> for TFunc
// {
//     type Fut = TFut;

//     fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut {
//         (self)()
//     }
// }

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

// // TODO: probs remove this once `MwV2` and `Executable` are merged
// pub struct IntoExecutable<TLCtx, TMarker, T>(
//     pub(crate) T,
//     pub(crate) PhantomData<(TLCtx, TMarker)>,
// )
// where
//     TLCtx: Send + 'static,
//     TMarker: Send + 'static,
//     T: MwV2<TLCtx, TMarker>;

// impl<TLCtx, TMarker, T> Executable<TLCtx, TMarker, T::Result> for IntoExecutable<TLCtx, TMarker, T>
// where
//     TLCtx: Send + 'static,
//     TMarker: Send + 'static,
//     T: MwV2<TLCtx, TMarker>,
// {
//     type Fut = <T as MwV2<TLCtx, TMarker>>::Fut;

//     fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TMarker) -> Self::Fut {
//         todo!()
//     }
// }
