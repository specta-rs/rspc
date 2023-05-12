use super::{
    AlphaMiddlewareBuilderLike, AlphaProcedure, AlphaRequestLayer, FutureMarker,
    RequestLayerMarker, ResolverFunction, StreamLayerMarker, StreamMarker,
};

/// TODO
// TODO: Rename cause this trait is exposed to userspace
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
        R::Result: AlphaRequestLayer<R::RequestMarker, Type = FutureMarker>;

    fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, Type = FutureMarker>;

    fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, StreamLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, Type = StreamMarker>;
}
