use std::marker::PhantomData;

use futures::Stream;
use serde_json::Value;

use crate::{
    internal::{RequestContext, SealedLayer},
    ExecError,
};

pub struct ResolverLayer<TLayerCtx, T, S>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
    S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx, S> SealedLayer<TLayerCtx> for ResolverLayer<TLayerCtx, T, S>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
    S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
{
    type Stream<'a> = S;

    fn call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<Self::Stream<'a>, ExecError> {
        (self.func)(a, b, c)
    }
}
