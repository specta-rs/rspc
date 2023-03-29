use std::{future::Future, marker::PhantomData, pin::Pin};

use serde_json::Value;

use crate::{
    internal::{LayerResult, RequestContext, ValueOrStream},
    ExecError,
};

// TODO: Rename this so it doesn't conflict with the middleware builder struct
pub trait AlphaLayer<TLayerCtx: 'static>: DynLayer<TLayerCtx> + Send + Sync + 'static {
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError>;

    fn erase(self) -> Box<dyn DynLayer<TLayerCtx>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// TODO: Does this need lifetime?
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
        // Ok(Box::pin(async move {
        //     match AlphaLayer::call(self, a, b, c).await? {
        //         ValueOrStream::Value(x) => Ok(ValueOrStream::Value(x)),
        //         ValueOrStream::Stream(x) => Ok(ValueOrStream::Stream(x)),
        //     }
        // }))
        todo!();
    }
}
