use std::{future::Future, marker::PhantomData};

use serde_json::Value;

use crate::{
    alpha::{MiddlewareArgMapper, MiddlewareArgMapperPassthrough},
    internal::RequestContext,
};

use super::{Executable2Placeholder, MwResultWithCtx, MwV2Result};

pub trait MwV2<TLCtx, TMarker: Send>: Send + 'static {
    type Fut: Future<Output = Self::Result>;
    type Result: MwV2Result;
    type NewCtx: Send + Sync + 'static;
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
}

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
