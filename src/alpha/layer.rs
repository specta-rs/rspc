use std::{future::Future, pin::Pin};

use serde_json::Value;

use crate::{
    internal::{RequestContext, ValueOrStream},
    ExecError,
};

pub trait AlphaLayer<TLayerCtx: 'static>: DynLayer<TLayerCtx> + Send + Sync + 'static {
    type Fut<'a>: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'a;

    fn call<'a>(&'a self, a: TLayerCtx, b: Value, c: RequestContext) -> Self::Fut<'a>;

    fn erase(self) -> Box<dyn DynLayer<TLayerCtx>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// TODO: Make this an enum so it can be `Value || Stream`?
pub type FutureValueOrStream<'a> =
    Pin<Box<dyn Future<Output = Result<ValueOrStream, ExecError>> + Send + 'a>>;

pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn dyn_call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream<'a>, ExecError>;
}

impl<TLayerCtx: Send + 'static, L: AlphaLayer<TLayerCtx>> DynLayer<TLayerCtx> for L {
    fn dyn_call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream<'a>, ExecError> {
        Ok(Box::pin(AlphaLayer::call(self, a, b, c)))
    }
}
