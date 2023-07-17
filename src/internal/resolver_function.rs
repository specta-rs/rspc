use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use super::RequestLayer;

#[doc(hidden)]
pub trait ResolverFunction<TLCtx, TMarker>:
    SealedResolverFunction<TMarker> + Fn(TLCtx, Self::Arg) -> Self::Result
{
}

mod private {
    use crate::internal::middleware::ProcedureKind;

    use super::*;

    // TODO: Rename
    pub trait SealedResolverFunction<TMarker>: Send + Sync + 'static {
        // TODO: Can a bunch of these assoicated types be removed?

        type Arg: DeserializeOwned + Type + 'static;
        type RequestMarker;
        type Result;

        fn into_marker(self, kind: ProcedureKind) -> TMarker;
    }

    pub struct HasResolver<TResolver, TArg, TResult, TResultMarker, TMiddleware> {
        pub(crate) resolver: TResolver,
        pub(crate) kind: ProcedureKind,
        _phantom: PhantomData<(TArg, TResult, TResultMarker, TMiddleware)>,
    }

    impl<TResolver, TArg, TResult, TResultMarker, TMiddleware>
        HasResolver<TResolver, TArg, TResult, TResultMarker, TMiddleware>
    {
        pub fn new(kind: ProcedureKind, resolver: TResolver) -> Self {
            Self {
                resolver,
                kind,
                _phantom: PhantomData,
            }
        }
    }

    impl<
            TMarker,
            TLCtx,
            T: SealedResolverFunction<TMarker> + Fn(TLCtx, Self::Arg) -> Self::Result,
        > ResolverFunction<TLCtx, TMarker> for T
    {
    }

    // TODO: This is always `RequestLayerMarker` which breaks shit

    // TODO: Remove TResultMarker

    impl<F, TLayerCtx, TArg, TResult, TResultMarker>
        SealedResolverFunction<HasResolver<F, TArg, TResult, TResultMarker, TLayerCtx>> for F
    where
        F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
        TArg: DeserializeOwned + Type + 'static,
        TResult: RequestLayer<TResultMarker>,
        TLayerCtx: Send + Sync + 'static,
    {
        type Arg = TArg;
        type RequestMarker = TResultMarker;
        type Result = TResult;

        fn into_marker(
            self,
            kind: ProcedureKind,
        ) -> HasResolver<F, TArg, TResult, TResultMarker, TLayerCtx> {
            HasResolver::new(kind, self)
        }
    }
}

pub(crate) use private::{HasResolver, SealedResolverFunction};
