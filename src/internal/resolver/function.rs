use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use super::RequestLayer;

mod private {
    use crate::internal::middleware::ProcedureKind;

    use super::*;

    pub trait ResolverFunction<TLCtx, TMarker>:
        Fn(TLCtx, Self::Arg) -> Self::Result + Send + Sync + 'static
    {
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

    // TODO: Expand all generic names cause they probs will show up in user-facing compile errors

    const _: () = {
        impl<TLayerCtx, TArg, TResult, TResultMarker, F>
            ResolverFunction<TLayerCtx, HasResolver<F, TLayerCtx, TArg, TResult, TResultMarker>>
            for F
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
    };
}

pub(crate) use private::{HasResolver, ResolverFunction};
