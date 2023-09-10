use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

mod private {
    use std::borrow::Cow;

    use futures::Stream;
    use serde::Serialize;
    use serde_json::Value;
    use specta::{ts::TsExportError, TypeMap};

    use crate::{
        internal::{
            middleware::{ProcedureKind, RequestContext},
            procedure::ProcedureDef,
            resolver::IntoQueryMutationResponse,
        },
        IntoResolverError,
    };

    use super::*;

    /// TODO
    pub trait ResolverFunction<TLCtx>: Send + Sync + 'static {
        // TODO: How da hell if this needs to be dyn-safe. It needs to end up boxed too, basically but then that prevents boxing anything refering to it
        // type Stream<'a>: Body + Send + 'a;

        fn into_procedure_def(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, TsExportError>;

        // TODO: The return type can't be `Value` cause streams and stuff
        fn exec(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Value;
    }

    // TODO: Allow transforming into a boxed variant of the function

    // TODO: Rename `Resolver`?
    pub struct HasResolver<F, TResult, M> {
        resolver: F,
        pub(crate) kind: ProcedureKind,
        phantom: PhantomData<fn() -> (TResult, M)>,
    }

    impl<F, TResult, M> HasResolver<F, TResult, M> {
        pub(crate) fn new(resolver: F, kind: ProcedureKind) -> Self {
            Self {
                resolver,
                kind,
                phantom: PhantomData,
            }
        }
    }

    pub struct M<TArg>(PhantomData<TArg>);
    impl<F, TLCtx, TResult, TArg> ResolverFunction<TLCtx> for HasResolver<F, TResult, M<TArg>>
    where
        F: Fn(TLCtx, TArg) -> TResult + Send + Sync + 'static,
        TArg: DeserializeOwned + Type + 'static,
        TLCtx: Send + Sync + 'static,
        TResult: 'static,
    {
        // type Stream<'a> = Once<Ready<Result<Value, ExecError>>>;

        fn into_procedure_def(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, TsExportError> {
            // ProcedureDef::from_tys::<TArg, TOk, TError>(key, ty_store)

            // TODO: Fix this
            ProcedureDef::from_tys::<TArg, (), ()>(key, ty_store)
        }

        fn exec(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Value {
            // TODO: Error handling
            // serde_json::to_value((self.resolver)(ctx, serde_json::from_value(input).unwrap()))
            //     .unwrap()
            todo!();
        }
    }

    pub trait QueryMutationFn<TErr, M> {}

    impl<F, TResult, M, TMarker, TErr> QueryMutationFn<TErr, TMarker> for HasResolver<F, TResult, M>
    where
        TResult: IntoQueryMutationResponse<TMarker, TErr>,
        TResult::Ok: Serialize + Type + 'static,
    {
    }
}

pub(crate) use private::{HasResolver, QueryMutationFn, ResolverFunction};
