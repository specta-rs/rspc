use crate::{internal::ProcedureKind, RequestLayer, StreamRequestLayer};

use super::{
    AlphaBaseMiddleware, AlphaMiddlewareBuilderLike, AlphaProcedure, AlphaRequestLayer,
    AlphaStreamRequestLayer, RequestLayerMarker, ResolverFunction, StreamLayerMarker,
};

/// TODO
pub trait ProcedureLike {
    type Middleware: AlphaMiddlewareBuilderLike<LayerCtx = Self::LayerCtx>;
    type LayerCtx: Send + Sync + 'static;

    fn query<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker>;

    fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker>;

    fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, StreamLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaStreamRequestLayer<R::RequestMarker>;
}
