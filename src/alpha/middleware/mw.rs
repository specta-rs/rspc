use std::{future::Future, marker::PhantomData};

use serde_json::Value;

use crate::{
    alpha::{MiddlewareArgMapper, MiddlewareArgMapperPassthrough},
    internal::RequestContext,
};

use super::{Executable, Executable2Placeholder, MwResultWithCtx, MwV2Result};

// TODO: Maybe deprecate this trait in favor of `Executable`?
pub trait MwV2<TLCtx, TMarker: Send>: Send + 'static {
    type Fut: Future<Output = Self::Result>;
    type Result: MwV2Result;
    type NewCtx: Send + Sync + 'static;

    // TODO: Take in `AlphaMiddlewareContext` directly???
    fn exec(
        &self,
        ctx: Self::NewCtx,
        input: Value,
        req: RequestContext,
        state: <<Self::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
    ) -> Self::Fut;
}

// TODO: Maybe replace with executable?
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

    fn exec(
        &self,
        ctx: Self::NewCtx,
        input: Value,
        req: RequestContext,
        state: <R::MwMapper as MiddlewareArgMapper>::State,
    ) -> Self::Fut {
        (self)(
            AlphaMiddlewareContext {
                input,
                req,
                state,
                _priv: (),
            },
            ctx,
        )
    }
}

// impl<TLCtx, F, Fu, R>
//     Executable<TLCtx, <R::MwMapper as MiddlewareArgMapper, MwV2Marker<(), ()>>::State, R::ActualOutput> for F
// where
//     TLCtx: Send + Sync + 'static,
//     F: Fn(AlphaMiddlewareContext<<R::MwMapper as MiddlewareArgMapper>::State>, TLCtx) -> Fu
//         + Send
//         + 'static,
//     Fu: Future<Output = R> + Send + 'static,
//     R: MwV2Result + Send + 'static,
// {
//     type Fut = Fu;

//     fn call(
//         &self,
//         ctx: TLCtx,
//         input: Value,
//         req: RequestContext,
//         state: <R::MwMapper as MiddlewareArgMapper>::State,
//     ) -> Self::Fut {
//         todo!()
//     }
// }

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

// TODO: Only hold output and not the whole `M` generic?
pub struct AlphaMiddlewareContext<MState> {
    pub input: Value,
    pub req: RequestContext,
    pub state: MState,
    _priv: (),
}

impl<MState> AlphaMiddlewareContext<MState> {
    pub fn next<TNCtx>(
        self,
        ctx: TNCtx,
    ) -> MwResultWithCtx<TNCtx, MiddlewareArgMapperPassthrough, Executable2Placeholder> {
        MwResultWithCtx {
            ctx,
            resp: None,
            phantom: PhantomData,
        }
    }

    pub fn args<M2: MiddlewareArgMapper>(self) -> AlphaMiddlewareContext2<M2> {
        AlphaMiddlewareContext2 {
            input: self.input,
            req: self.req,
            phantom: PhantomData,
        }
    }
}

pub struct AlphaMiddlewareContext2<M> {
    input: Value,
    req: RequestContext,
    phantom: PhantomData<M>,
}

impl<M> AlphaMiddlewareContext2<M>
where
    M: MiddlewareArgMapper,
{
    pub fn next<TNCtx>(self, ctx: TNCtx) -> MwResultWithCtx<TNCtx, M, Executable2Placeholder> {
        MwResultWithCtx {
            ctx,
            resp: None,
            phantom: PhantomData,
        }
    }
}
