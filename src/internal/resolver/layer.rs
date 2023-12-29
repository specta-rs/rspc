//! Move this into the `layer` module instead of the `resolver` module???

use std::pin::Pin;

use futures::Stream;
use serde_json::Value;

use crate::{error::ExecError, internal::middleware::RequestContext, layer::Layer};

type ErasedLayerFn<TLCtx> = Box<
    dyn Fn(
            TLCtx,
            Value,
            RequestContext,
        ) -> Result<
            Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'static>>,
            ExecError,
        > + Send
        + Sync
        + 'static,
>;

pub(crate) struct LayerFn<F>(F);

impl<F> LayerFn<F> {
    pub(crate) fn new<TLCtx, S>(f: F) -> Self
    where
        TLCtx: 'static,
        F: Fn(TLCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
        S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
    {
        Self(f)
    }

    pub(crate) fn erased<TLCtx, S>(self) -> LayerFn<ErasedLayerFn<TLCtx>>
    where
        TLCtx: 'static,
        F: Fn(TLCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
        S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
    {
        LayerFn(Box::new(
            move |ctx: TLCtx, input: Value, req: RequestContext| {
                Ok(Box::pin((self.0)(ctx, input, req)?))
            },
        ))
    }
}

impl<TLCtx, F, S> Layer<TLCtx> for LayerFn<F>
where
    TLCtx: 'static,
    F: Fn(TLCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
    S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
{
    type Stream<'a> = S;

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        (self.0)(ctx, input, req)
    }
}
