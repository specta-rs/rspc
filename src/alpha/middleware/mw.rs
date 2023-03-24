use std::{
    future::{Future, Ready},
    marker::PhantomData,
};

use serde_json::Value;

use crate::{
    alpha::{MiddlewareArgMapper, MiddlewareArgMapperPassthrough},
    internal::RequestContext,
};

use super::{
    AlphaMiddlewareContext, Demo, Executable, Executable2Placeholder, MwResultWithCtx, MwV2Result,
};

pub trait MwV2<TLCtx, TMarker: Send>: Send + 'static {
    type Fut: Future<Output = Self::Result>;
    type Result: MwV2Result;
    type NewCtx: Send + Sync + 'static;

    type Executable: Executable<
        TLCtx,
        <<Self::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
        Value,
        (),
    >;

    fn into_executable(self) -> Self::Executable;
}

pub struct MwV2Marker<A, B>(PhantomData<(A, B)>);
impl<TLCtx, F, Fu, R> MwV2<TLCtx, MwV2Marker<Fu, R>> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(AlphaMiddlewareContext<<R::MwMapper as MiddlewareArgMapper>::State>, TLCtx) -> Fu
        + Send
        + 'static,
    Fu: Future<Output = R> + Send + 'static,
    R: MwV2Result + Send + 'static,
{
    type Fut = Fu;
    type Result = R;
    type NewCtx = TLCtx; // TODO: Make this work with context switching

    type Executable = Demo<TLCtx, <R::MwMapper as MiddlewareArgMapper>::State, Value>; // TODO: Custom type here

    fn into_executable(self) -> Self::Executable {
        todo!();
    }
}

// pub struct ExecutableMwV2<TLCtx, TMarker: Send, T: MwV2<TLCtx, TMarker>>(
//     T,
//     PhantomData<(TLCtx, TMarker)>,
// );

// impl<TLCtx, TMarker: Send, T: MwV2<TLCtx, TMarker>>
//     Executable<TLCtx, <R::MwMapper as MiddlewareArgMapper>::State, R::>
//     for ExecutableMwV2<TLCtx, TMarker, T>
// {
//     type Fut;

//     fn call(&self, ctx: TLCtx, input: Value, req: RequestContext, state: TState) -> Self::Fut {
//         todo!()
//     }
// }
