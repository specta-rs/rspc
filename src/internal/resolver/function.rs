use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

mod private {
    use std::borrow::Cow;

    use serde::Serialize;
    use serde_json::Value;
    use specta::{ts::TsExportError, TypeMap};

    use crate::{
        internal::{
            middleware::{MiddlewareBuilder, ProcedureKind, RequestContext},
            procedure::ProcedureDef,
        },
        IntoResolverError,
    };

    use super::*;

    // TODO: Allow transforming into a boxed variant of the function

    // TODO: Docs + rename cause it's not a marker, it's runtime
    // TODO: Can this be done better?
    // TODO: Remove `TLCtx` from this - It's being used to contain stuff but there would be a better way
    // TODO: Remove `TError` from this
    pub struct HasResolver<F, M> {
        resolver: F,
        pub(crate) kind: ProcedureKind,
        phantom: PhantomData<fn() -> M>,
    }

    impl<F, M> HasResolver<F, M> {
        pub fn new(resolver: F, kind: ProcedureKind) -> Self {
            Self {
                resolver,
                kind,
                phantom: PhantomData,
            }
        }
    }

    // TODO: If this stays around can it be `pub(crate)`???
    pub trait ResolverFunction<TLCtx, TError>: Send + Sync + 'static {
        // type Stream<'a>: Body + Send + 'a;

        fn into_procedure_def<TMiddleware: MiddlewareBuilder>(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, TsExportError>;

        // TODO: The return type can't be `Value` cause streams and stuff
        fn exec(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Value;
    }

    // TODO: `M` being hardcoded -> maybe not?
    impl<F, TLCtx, TArg, TOk, TError> ResolverFunction<TLCtx, TError>
        for HasResolver<F, M<TArg, TOk, TError>>
    where
        F: Fn(TLCtx, TArg) -> Result<TOk, TError> + Send + Sync + 'static,
        TArg: DeserializeOwned + Type + 'static,
        TOk: Serialize + Type + 'static,
        TError: IntoResolverError + 'static,
        TLCtx: Send + Sync + 'static,
    {
        // type Stream<'a> = Once<Ready<Result<Value, ExecError>>>;

        fn into_procedure_def<TMiddleware: MiddlewareBuilder>(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, TsExportError> {
            ProcedureDef::from_tys::<TMiddleware::Arg<TArg>, TOk, TError>(key, ty_store)
        }

        fn exec(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Value {
            // TODO: Error handling
            serde_json::to_value((self.resolver)(ctx, serde_json::from_value(input).unwrap()))
                .unwrap()
        }
    }

    // TODO: move into `const` blocks
    pub struct M<TArg, TOk, TError>(PhantomData<(TArg, TOk, TError)>);

    // TODO: Expand all generic names cause they probs will show up in user-facing compile errors

    // TODO: Finish off the rest of the impls once stuff is sorted out a bit.
}

pub(crate) use private::{HasResolver, ResolverFunction};
