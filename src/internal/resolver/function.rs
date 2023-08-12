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

    // TODO: Docs + rename cause it's not a marker, it's runtime
    pub struct HasResolver<A, B, C, D, E>(
        pub(crate) A,
        pub(crate) ProcedureKind,
        pub(crate) PhantomData<(B, C, D, E)>,
    );

    impl<
            TMarker,
            TLCtx,
            T: SealedResolverFunction<TMarker> + Fn(TLCtx, Self::Arg) -> Self::Result,
        > ResolverFunction<TLCtx, TMarker> for T
    {
    }

    // TODO: This is always `RequestLayerMarker` which breaks shit

    // TODO: Remove TResultMarker

    impl<TLayerCtx, TArg, TResult, TResultMarker, F>
        SealedResolverFunction<HasResolver<F, TLayerCtx, TArg, TResult, TResultMarker>> for F
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
        ) -> HasResolver<F, TLayerCtx, TArg, TResult, TResultMarker> {
            HasResolver(self, kind, PhantomData)
        }
    }
}

pub(crate) use private::{HasResolver, SealedResolverFunction};
