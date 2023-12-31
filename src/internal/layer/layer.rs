use futures::{stream::once, Stream};
use serde_json::Value;
use std::{future::ready, pin::Pin};

use crate::{error::ExecError, internal::middleware::RequestContext};

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<impl Stream<Item = Result<Value, ExecError>> + Send + 'static, ExecError>;
}

// TODO: Replace this with `rspc_core::Procedure` if possible
pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'static>>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'static>> {
        match self.call(ctx, input, req) {
            Ok(stream) => Box::pin(stream),
            // TODO: Avoid allocating error future here
            Err(err) => Box::pin(once(ready(Err(err)))),
        }
    }
}
