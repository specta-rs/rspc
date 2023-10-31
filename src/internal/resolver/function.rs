use std::{borrow::Cow, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use specta::{ts, Type, TypeMap};

use rspc_core::internal::{IntoResolverError, Layer, ProcedureDef, ProcedureKind, RequestContext};

use crate::{
    internal::resolver::{result::private::StreamToBody, IntoResolverResponse},
    ExecError,
};

pub struct QueryOrMutation<M>(PhantomData<M>);
pub struct Subscription<M>(PhantomData<M>);

// TODO: Rename `Resolver`?
pub struct HasResolver<F, TErr, TResultMarker, M> {
    pub(crate) resolver: F,
    pub(crate) kind: ProcedureKind,
    phantom: PhantomData<fn() -> (TErr, TResultMarker, M)>,
}

mod private {
    use super::*;

    impl<F, TErr, TResultMarker, M> HasResolver<F, TErr, TResultMarker, M> {
        pub(crate) fn new(resolver: F, kind: ProcedureKind) -> Self {
            Self {
                resolver,
                kind,
                phantom: PhantomData,
            }
        }
    }

    pub struct M<TArg>(PhantomData<TArg>);
    impl<F, TLCtx, TArg, TResult, TResultMarker> Layer<TLCtx>
        for HasResolver<F, TResult::Err, TResultMarker, M<TArg>>
    where
        F: Fn(TLCtx, TArg) -> TResult + Send + Sync + 'static,
        TArg: DeserializeOwned + Type + 'static,
        TLCtx: Send + Sync + 'static,
        TResult: IntoResolverResponse<'static, TResultMarker>,
        TResult::Ok: Serialize + Type + 'static,
        TResult::Err: IntoResolverError + 'static,
        TResultMarker: 'static,
    {
        type Stream<'a> = StreamToBody<TResult::Stream>;

        fn into_procedure_def(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, ts::ExportError> {
            ProcedureDef::from_tys::<TArg, TResult::Ok, TResult::Err>(key, ty_store)
        }

        fn call(
            &self,
            ctx: TLCtx,
            input: Value,
            req: RequestContext,
        ) -> Result<Self::Stream<'_>, ExecError> {
            let stream = (self.resolver)(
                ctx,
                serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
            )
            .to_stream();

            Ok(StreamToBody {
                stream,
                #[cfg(feature = "tracing")]
                span: req.span(),
                #[cfg(not(feature = "tracing"))]
                span: None,
            })
        }
    }
}
