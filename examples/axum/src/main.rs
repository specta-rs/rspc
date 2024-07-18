use std::net::Ipv6Addr;

use api::invalidation;
use axum::{routing::get, Router};
use tokio::sync::broadcast;
use tracing::info;

mod api;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = api::mount().build().unwrap();

    let chat_tx = broadcast::channel(100).0;
    let invalidation = invalidation::Ctx::new();
    let ctx_fn = move || api::Context {
        chat: api::chat::Ctx::new(chat_tx.clone()),
        invalidation: invalidation.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/rspc",
            rspc_axum::Endpoint::new(router.clone(), ctx_fn.clone()),
        )
        .nest("/", rspc_openapi::mount(router, ctx_fn));

    info!("Listening on http://[::1]:3000");
    let listener = tokio::net::TcpListener::bind((Ipv6Addr::UNSPECIFIED, 3000))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
