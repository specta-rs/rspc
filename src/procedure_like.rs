use crate::internal::{
    middleware::MiddlewareBuilder, procedure::Procedure, FutureMarkerType, RequestLayer,
    RequestLayerMarker, ResolverFunction, SealedRequestLayer, StreamLayerMarker, StreamMarkerType,
};

// TODO: Seal and move into `internal/procedure`

/// TODO
// TODO: Rename cause this trait is exposed to userspace
pub trait ProcedureLike {
    type Middleware: MiddlewareBuilder<LayerCtx = Self::LayerCtx>;
    type LayerCtx: Send + Sync + 'static;

    fn query<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, Self::Middleware, RequestLayerMarker<RMarker>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>;

    fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, Self::Middleware, RequestLayerMarker<RMarker>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>;

    fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, Self::Middleware, StreamLayerMarker<RMarker>>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = Self::LayerCtx>
            + Fn(Self::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>;
}
