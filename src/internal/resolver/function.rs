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

    // TODO: If this stays around can it be `pub(crate)`???
    pub trait ResolverFunctionGood<TLCtx, TError>: Send + Sync + 'static {
        // type Stream<'a>: Body + Send + 'a;

        fn into_procedure_def<TMiddleware: MiddlewareBuilder>(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, TsExportError>;

        // TODO: The return type can't be `Value` cause streams and stuff
        fn exec(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Value;
    }

    // TODO: Allow transforming `ResolverFunctionGood` into a boxed variant

    // TODO: `M` being hardcoded -> maybe not?
    impl<F, TLCtx, TArg, TOk, TError> ResolverFunctionGood<TLCtx, TError>
        for HasResolver<F, TLCtx, TError, M<TArg, TOk, TError>>
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
            serde_json::to_value((self.0)(ctx, serde_json::from_value(input).unwrap())).unwrap()
        }
    }

    // TODO: dyn-erase types at barrier of this
    pub trait ResolverFunction<TLCtx, TError, TMarker>:
        Fn(TLCtx, Self::Arg) -> Self::Result + Send + Sync + 'static
    {
        // TODO: Can all of these assoicated types be removed?
        type Arg: DeserializeOwned + Type + 'static;
        type Result;

        // TODO: Make `&self`?
        fn into_marker(self, kind: ProcedureKind) -> TMarker;
    }

    // TODO: Renamed struct
    // TODO: Docs + rename cause it's not a marker, it's runtime
    // TODO: Can this be done better?
    // TODO: Remove `TLCtx` from this - It's being used to contain stuff but there would be a better way
    // TODO: Remove `TError` from this
    pub struct HasResolver<F, TLCtx, TError, M>(
        pub(crate) F,
        pub(crate) ProcedureKind,
        pub(crate) PhantomData<fn() -> (TLCtx, M, TError)>,
    );

    // TODO: move into `const` blocks
    pub struct M<TArg, TOk, TError>(PhantomData<(TArg, TOk, TError)>);

    // TODO: Expand all generic names cause they probs will show up in user-facing compile errors

    // Result<_, _>
    const _: () = {
        impl<TLayerCtx, TArg, F, TOk, TError>
            ResolverFunction<
                TLayerCtx,
                TError,
                HasResolver<F, TLayerCtx, TError, M<TArg, TOk, TError>>,
            > for F
        where
            F: Fn(TLayerCtx, TArg) -> Result<TOk, TError> + Send + Sync + 'static,
            TArg: DeserializeOwned + Type + 'static,
            TOk: Serialize + Type,
            TError: IntoResolverError,
            TLayerCtx: Send + Sync + 'static,
        {
            type Arg = TArg;
            type Result = Result<TOk, TError>;

            fn into_marker(
                self,
                kind: ProcedureKind,
            ) -> HasResolver<F, TLayerCtx, TError, M<TArg, TOk, TError>> {
                HasResolver(self, kind, PhantomData)
            }
        }
    };

    // TODO: Finish off the rest of the impls once stuff is sorted out a bit.
}

pub(crate) use private::{HasResolver, ResolverFunction, ResolverFunctionGood};
