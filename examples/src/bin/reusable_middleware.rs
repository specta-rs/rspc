//!
//! This API is currently very hard to work with and is without a doubt going to improve in future release.
//! It also relies on `rspc::internal` types which may change in between releases.
//!

use axum::routing::get;
use rspc::{
    internal::{MiddlewareBuilder, MiddlewareLike},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

fn logger_middleware(
    mw: MiddlewareBuilder<()>,
) -> impl MiddlewareLike<(), NewCtx = ()> + Send + Sync + 'static {
    mw.middleware(|mw| async move {
        let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
        Ok(mw.with_state(state))
    })
    .resp(|state, result| async move {
        println!(
            "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
            state.0, state.1, state.2, result
        );
        Ok(result)
    })
}

#[tokio::main]
async fn main() {
    let router = Router::<()>::new()
        .middleware(logger_middleware)
        .query("version", |t| {
            t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") })
        })
        .build()
        .arced();

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest("/rspc", router.endpoint(|| ()).axum())
        // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
