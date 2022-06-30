use std::sync::Arc;

use axum::routing::get;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = rspc::Router::<()>::new()
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .query("getUser", |_, v: String| v);
    let router = Arc::new(router.build());

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route("/trpc/:id", router.axum_handler(|| ()))
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!(
        "listening on http://{}/trpc/version?batch=1&input=%7B%7D",
        addr
    );
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
