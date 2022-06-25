use std::{path::Path, sync::Arc};

use axum::routing::get;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = trpc_rs::Router::<()>::new()
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .query("getUser", |_, v: String| v);
    let router = Arc::new(router);

    router.export(Path::new("./ts")).unwrap();

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'trpc-rs'!" }))
        .route("/trpc/:id", router.axum_handler(|| ()))
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
