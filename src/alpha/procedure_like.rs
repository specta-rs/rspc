use super::{
    AlphaMiddlewareBuilderLike, AlphaProcedure, AlphaRequestLayer, FutureMarker,
    RequestLayerMarker, ResolverFunction, StreamLayerMarker, StreamMarker,
};

/// TODO
// TODO: Rename cause this trait is exposed to userspace
pub trait ProcedureLike {
    type Middleware: AlphaMiddlewareBuilderLike<LayerCtx = Self::LayerCtx>;
    type LayerCtx: Send + Sync + 'static;
    type Error;

    fn query<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, Self::Error, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, Self::Error, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, Self::Error, Type = FutureMarker>;

    fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, Self::Error, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, Self::Error, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, Self::Error, Type = FutureMarker>;

    fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, Self::Error, StreamLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, Self::Error, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, Self::Error, Type = StreamMarker>;
}
