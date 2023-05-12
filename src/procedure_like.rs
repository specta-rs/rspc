use crate::{
    internal::{
        FutureMarkerType, RequestLayer, RequestLayerMarker, ResolverFunction, SealedRequestLayer,
        StreamLayerMarker, StreamMarkerType,
    },
    MiddlewareBuilderLike, Procedure,
};

/// TODO
// TODO: Rename cause this trait is exposed to userspace
pub trait ProcedureLike {
    type Middleware: MiddlewareBuilderLike<LayerCtx = Self::LayerCtx>;
    type LayerCtx: Send + Sync + 'static;

    fn query<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>;

    fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>;

    fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, StreamLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>;
}
