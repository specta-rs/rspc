use axum::routing::get;
use rspc::BuiltRouter;

pub struct Endpoint<TCtx> {
    router: BuiltRouter<TCtx>,
}

impl<TCtx> Endpoint<TCtx> {
    // TODO: Async context function
    pub fn new(router: BuiltRouter<TCtx>, ctx_fn: impl Fn() -> TCtx) -> Self {
        Self { router }
    }

    // TODO: `Clone` bounds for websockets
    // TODO: Configuration???

    pub fn build<S: Clone + Send + Sync + 'static>(self) -> axum::Router<S> {
        axum::Router::new().route("/todo", get(|| async move { "this is rspc" }))
    }
}
