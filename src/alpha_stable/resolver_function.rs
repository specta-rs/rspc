use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use specta::{ts::TsExportError, DefOpts, Type, TypeDefs};

use crate::{
    alpha::{AlphaRequestLayer, AlphaStreamRequestLayer},
    internal::ProcedureDataType,
    RequestLayer, StreamRequestLayer,
};

use super::{RequestLayerMarker, StreamLayerMarker};

pub trait ResolverFunction<TMarker>: Send + Sync + 'static {
    type LayerCtx: Send + Sync + 'static;
    type Arg: DeserializeOwned + Type;
    type RequestMarker;
    type Result;
    type ResultMarker;

    type RawResult: Type; // TODO: Can we remove this. It's basically `Self::Result`

    fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result;

    fn typedef(
        key: Cow<'static, str>,
        defs: &mut TypeDefs,
    ) -> Result<ProcedureDataType, TsExportError> {
        Ok(ProcedureDataType {
            key,
            input: <Self::Arg as Type>::reference(
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

pub struct Marker<A, B, C, D>(PhantomData<(A, B, C, D)>);

impl<
        TLayerCtx,
        TArg,
        TResult,
        TResultMarker,
        F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    > ResolverFunction<RequestLayerMarker<Marker<TArg, TResult, TResultMarker, TLayerCtx>>> for F
where
    TArg: DeserializeOwned + Type,
    TResult: AlphaRequestLayer<TResultMarker>,
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
    TArg: DeserializeOwned + Type,
    TResult: AlphaStreamRequestLayer<TResultMarker>,
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

impl<TLayerCtx> Default for MissingResolver<TLayerCtx> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}
