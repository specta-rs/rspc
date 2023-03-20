use crate::{internal::ProcedureKind, RequestLayer, StreamRequestLayer};

use super::{AlphaBaseMiddleware, AlphaMiddlewareBuilderLike, AlphaProcedure, ResolverFunction};

/// TODO
pub trait ProcedureLike<TLayerCtx: Send + Sync + 'static> {
    type Middleware: AlphaMiddlewareBuilderLike<LayerContext = TLayerCtx> + Send;

    fn query<R, RMarker>(self, builder: R) -> AlphaProcedure<R, RMarker, Self::Middleware>
    where
        R: ResolverFunction<RMarker, LayerCtx = TLayerCtx> + Fn(TLayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RawResultMarker>;

    fn mutation<R, RMarker>(self, builder: R) -> AlphaProcedure<R, RMarker, Self::Middleware>
    where
        R: ResolverFunction<RMarker, LayerCtx = TLayerCtx> + Fn(TLayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RawResultMarker>;

    fn subscription<R, RMarker>(self, builder: R) -> AlphaProcedure<R, RMarker, Self::Middleware>
    where
        R: ResolverFunction<RMarker, LayerCtx = TLayerCtx> + Fn(TLayerCtx, R::Arg) -> R::Result,
        R::Result: StreamRequestLayer<R::RawResultMarker>;
}
