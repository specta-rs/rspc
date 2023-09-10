//! TODO
//!
//! This module is sealed.

use std::future::ready;

use serde_json::Value;

use super::Body;
use crate::internal::middleware::RequestContext;

use crate::{internal::Once, ExecError};

pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
        // TODO: Return `Result` too?
    ) -> Box<dyn Body + Send + '_>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn dyn_call(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Box<dyn Body + Send + '_> {
        match self.call(ctx, input, req) {
            Ok(stream) => Box::new(stream),
            // TODO: Avoid allocating error future here
            Err(err) => Box::new(Once::new(ready(Err(err)))),
        }
    }
}

impl<TLCtx: Send + 'static> Layer<TLCtx> for Box<dyn DynLayer<TLCtx>> {
    type Stream<'a> = Box<dyn Body + Send + 'a>;

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        Ok(self.dyn_call(ctx, input, req))
    }
}

pub(crate) fn boxed<TLCtx: Send + 'static>(layer: impl Layer<TLCtx>) -> Box<dyn DynLayer<TLCtx>> {
    Box::new(layer)
}

/// Prevents the end user implementing the `Layer` trait and hides the internals
pub trait Layer<TLCtx: 'static>: Send + Sync + 'static {
    // TODO: Rename `Body`
    type Stream<'a>: Body + Send + 'a;

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError>;
}
