use crate::{internal::ProcedureKind, RequestLayer, StreamRequestLayer};

use super::{
    AlphaBaseMiddleware, AlphaMiddlewareBuilderLike, AlphaProcedure, RequestLayerMarker,
    ResolverFunction, StreamLayerMarker,
};

/// TODO
pub trait ProcedureLike<TLayerCtx: Send + Sync + 'static> {
    type Middleware: AlphaMiddlewareBuilderLike<LayerCtx = TLayerCtx>;

    fn query<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TLayerCtx>
            + Fn(TLayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>;

    fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TLayerCtx>
            + Fn(TLayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>;

    fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, StreamLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TLayerCtx>
            + Fn(TLayerCtx, R::Arg) -> R::Result,
        R::Result: StreamRequestLayer<R::RequestMarker>;
}
