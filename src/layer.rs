use futures::{future::ready, stream::once, Stream};
use serde_json::Value;
use specta::{ts, TypeMap};
use std::{borrow::Cow, pin::Pin};

use crate::{
    error::ExecError, middleware_from_core::RequestContext, procedure_store::ProcedureDef,
};

// TODO: Remove `SealedLayer`

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: Send + Sync + 'static {
    type Stream<'a>: Stream<Item = Result<Value, ExecError>> + Send + 'a;

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError>;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError>;
}

pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError>;

    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + '_>>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError> {
        Layer::into_procedure_def(self, key, ty_store)
    }

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

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError> {
        (&**self).into_procedure_def(key, ty_store)
    }

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
