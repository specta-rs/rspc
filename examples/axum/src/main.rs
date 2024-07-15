use std::net::Ipv6Addr;

use axum::{routing::get, Router};
use tracing::info;

mod api;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = api::mount().build().unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/rspc",
            rspc_axum::Endpoint::new(router, || api::Context {
                chat: Default::default(),
            })
            .with_endpoints()
            .with_websocket()
            .with_batching()
            .build(),
        );

    info!("Listening on http://[::1]:3000");
    let listener = tokio::net::TcpListener::bind((Ipv6Addr::UNSPECIFIED, 3000))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
