#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{net::SocketAddr, sync::Arc};
mod api;
mod utils;

use axum::{
    http::{HeaderValue, Method},
    routing::get,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");

    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let router = api::new().build().arced();
    let client = Arc::new(prisma::new_client().await.unwrap());

    let addr = "[::]:9000".parse::<SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("{} listening on http://{}", env!("CARGO_CRATE_NAME"), addr);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route(
            "/rspc/:id",
            router
                .endpoint(move || api::Ctx {
                    client: client.clone(),
                })
                .axum(),
        )
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(utils::axum_shutdown_signal())
        .await
        .expect("Error with HTTP server!");
}
