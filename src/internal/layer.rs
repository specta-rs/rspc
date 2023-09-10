//! TODO
//!
//! This module is sealed.

use std::borrow::Cow;
use std::future::ready;

use serde_json::Value;
use specta::ts::TsExportError;
use specta::TypeMap;

use super::procedure::ProcedureDef;
use super::Body;
use crate::internal::middleware::RequestContext;

use crate::{internal::Once, ExecError};

pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, TsExportError>;

    fn dyn_call(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Box<dyn Body + Send + '_>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, TsExportError> {
        Layer::into_procedure_def(self, key, ty_store)
    }

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

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, TsExportError> {
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

/// Prevents the end user implementing the `Layer` trait and hides the internals
pub trait Layer<TLCtx: 'static>: Send + Sync + 'static {
    // TODO: Rename `Body`
    type Stream<'a>: Body + Send + 'a;

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, TsExportError>;

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError>;
}
