use futures::{future::ready, stream::once, Stream};
use serde_json::Value;
use std::pin::Pin;

use crate::{error::ExecError, internal::middleware::RequestContext};

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: Send + Sync + 'static {
    type Stream<'a>: Stream<Item = Result<Value, ExecError>> + Send + 'a;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError>;
}

// TODO: Replace this with `rspc_core::Procedure` if possible
pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + '_>>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + '_>> {
        match self.call(ctx, input, req) {
            Ok(stream) => Box::pin(stream),
            // TODO: Avoid allocating error future here
            Err(err) => Box::pin(once(ready(Err(err)))),
        }
    }
}

impl<TLCtx: Send + 'static> Layer<TLCtx> for Box<dyn DynLayer<TLCtx>> {
    type Stream<'a> = Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'a>>;

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        Ok(self.dyn_call(ctx, input, req))
    }
}
