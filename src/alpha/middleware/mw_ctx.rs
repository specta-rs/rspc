use std::marker::PhantomData;

use serde_json::Value;

use crate::{
    alpha::{MiddlewareArgMapper, MiddlewareArgMapperPassthrough},
    internal::RequestContext,
};

use super::{Executable2Placeholder, MwResultWithCtx};

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
            ctx: Some(ctx),
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
            ctx: Some(ctx),
            resp: None,
            phantom: PhantomData,
        }
    }
}
