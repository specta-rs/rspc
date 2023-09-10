use std::{
    future::{ready, Ready},
    marker::PhantomData,
};

use serde::de::DeserializeOwned;
use specta::Type;

use std::borrow::Cow;

use serde::Serialize;
use serde_json::Value;
use specta::{ts::TsExportError, TypeMap};

use crate::{
    internal::{
        middleware::{ProcedureKind, RequestContext},
        procedure::ProcedureDef,
        resolver::IntoQueryMutationResponse,
        Layer, Once,
    },
    ExecError, IntoResolverError,
};

// TODO: Rename `Resolver`?
pub struct HasResolver<F, TErr, M> {
    resolver: F,
    pub(crate) kind: ProcedureKind,
    phantom: PhantomData<fn() -> (TErr, M)>,
}

impl<F, E, M> HasResolver<F, E, M> {
    pub(crate) fn new(resolver: F, kind: ProcedureKind) -> Self {
        Self {
            resolver,
            kind,
            phantom: PhantomData,
        }
    }
}

pub struct M<TArg, TResultMarker>(PhantomData<(TArg, TResultMarker)>);
impl<F, TLCtx, TErr, TArg, TResult, TResultMarker> Layer<TLCtx>
    for HasResolver<F, TErr, M<TArg, TResultMarker>>
where
    F: Fn(TLCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: DeserializeOwned + Type + 'static,
    TLCtx: Send + Sync + 'static,
    TResult: IntoQueryMutationResponse<TResultMarker, TErr>,
    TResult::Ok: Serialize + Type + 'static,
    TErr: IntoResolverError + 'static,
    TResultMarker: 'static,
{
    // TODO: This is a placeholder
    type Stream<'a> = Once<Ready<Result<Value, ExecError>>>;

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, TsExportError> {
        ProcedureDef::from_tys::<TArg, TResult::Ok, TErr>(key, ty_store)
    }

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        // TODO: Error handling
        let y = (self.resolver)(ctx, serde_json::from_value(input).unwrap());

        // TODO: Make this actually work
        Ok(Once::new(ready(Ok(Value::String("TODO".into())))))
    }
}
