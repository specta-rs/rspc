use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use specta::{ts::TsExportError, DefOpts, Type, TypeDefs};

use crate::internal::ProcedureDataType;

use super::{FutureMarker, RequestLayer, RequestLayerMarker, StreamLayerMarker, StreamMarker};

// TODO: private or sealed
pub trait AlphaMiddlewareBuilderLikeCompat {
    type Arg<T: Type + DeserializeOwned + 'static>: Type + DeserializeOwned + 'static;
}

#[doc(hidden)]
pub trait ResolverFunction<TMarker>: SealedResolverFunction<TMarker> {}

mod private {
    use crate::internal::SealedRequestLayer;

    use super::*;

    pub trait SealedResolverFunction<TMarker>: Send + Sync + 'static {
        type LayerCtx: Send + Sync + 'static;
        type Arg: DeserializeOwned + Type + 'static;
        type RequestMarker;
        type Result;
        type ResultMarker;

        type RawResult: Type; // TODO: Can we remove this. It's basically `Self::Result`

        fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result;

        fn typedef<TMiddleware: AlphaMiddlewareBuilderLikeCompat>(
            key: Cow<'static, str>,
            defs: &mut TypeDefs,
        ) -> Result<ProcedureDataType, TsExportError> {
            Ok(ProcedureDataType {
                key,
                input: <TMiddleware::Arg<Self::Arg> as Type>::reference(
                    DefOpts {
                        parent_inline: false,
                        type_map: defs,
                    },
                    &[],
                )?,
                result: <Self::RawResult as Type>::reference(
                    DefOpts {
                        parent_inline: false,
                        type_map: defs,
                    },
                    &[],
                )?,
            })
        }
    }

    impl<TMarker, T: SealedResolverFunction<TMarker>> ResolverFunction<TMarker> for T {}

    // TODO: Merge the following two impls? They are differentiated by `Type = X` but they have different markers through the rest of the system.

    pub struct Marker<A, B, C, D>(PhantomData<(A, B, C, D)>);

    impl<
            TLayerCtx,
            TArg,
            TResult,
            TResultMarker,
            F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
        >
        SealedResolverFunction<RequestLayerMarker<Marker<TArg, TResult, TResultMarker, TLayerCtx>>>
        for F
    where
        TArg: DeserializeOwned + Type + 'static,
        TResult:
            RequestLayer<TResultMarker> + SealedRequestLayer<TResultMarker, Type = FutureMarker>,
        TLayerCtx: Send + Sync + 'static,
    {
        type LayerCtx = TLayerCtx;
        type Arg = TArg;
        type Result = TResult;
        type ResultMarker = RequestLayerMarker<TResultMarker>;
        type RequestMarker = TResultMarker;
        type RawResult = TResult::Result;

        fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result {
            self(ctx, arg)
        }
    }

    impl<
            TLayerCtx,
            TArg,
            TResult,
            TResultMarker,
            F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
        >
        SealedResolverFunction<StreamLayerMarker<Marker<TArg, TResult, TResultMarker, TLayerCtx>>>
        for F
    where
        TArg: DeserializeOwned + Type + 'static,
        TResult:
            RequestLayer<TResultMarker> + SealedRequestLayer<TResultMarker, Type = StreamMarker>,
        TLayerCtx: Send + Sync + 'static,
    {
        type LayerCtx = TLayerCtx;
        type Arg = TArg;
        type Result = TResult;
        type ResultMarker = StreamLayerMarker<TResultMarker>;
        type RequestMarker = TResultMarker;
        type RawResult = TResult::Result;

        fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result {
            self(ctx, arg)
        }
    }
}

pub(crate) use private::SealedResolverFunction;
