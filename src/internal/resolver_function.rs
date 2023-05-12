use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use specta::{ts::TsExportError, DefOpts, Type, TypeDefs};

use crate::internal::ProcedureDataType;

use super::{AlphaRequestLayer, FutureMarker, RequestLayerMarker, StreamLayerMarker, StreamMarker};

pub trait AlphaMiddlewareBuilderLikeCompat {
    type Arg<T: Type + DeserializeOwned + 'static>: Type + DeserializeOwned + 'static;
}

pub trait ResolverFunction<TMarker>: Send + Sync + 'static {
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

// TODO: Merge the following two impls? They are differentiated by `Type = X` but they have different markers through the rest of the system.

pub struct Marker<A, B, C, D>(PhantomData<(A, B, C, D)>);

impl<
        TLayerCtx,
        TArg,
        TResult,
        TResultMarker,
        F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    > ResolverFunction<RequestLayerMarker<Marker<TArg, TResult, TResultMarker, TLayerCtx>>> for F
where
    TArg: DeserializeOwned + Type + 'static,
    TResult: AlphaRequestLayer<TResultMarker, Type = FutureMarker>,
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
    > ResolverFunction<StreamLayerMarker<Marker<TArg, TResult, TResultMarker, TLayerCtx>>> for F
where
    TArg: DeserializeOwned + Type + 'static,
    TResult: AlphaRequestLayer<TResultMarker, Type = StreamMarker>,
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

pub struct MissingResolver<TLayerCtx> {
    phantom: PhantomData<TLayerCtx>,
}

// TODO: Remove this and put the `MissingResolver` in phantom data if possible
impl<TLayerCtx> Default for MissingResolver<TLayerCtx> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}
