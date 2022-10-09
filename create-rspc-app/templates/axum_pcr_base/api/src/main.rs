use std::{net::SocketAddr, sync::Arc};
use axum::{
    http::{HeaderValue, Method},
    routing::get,
};
use tower_http::cors::CorsLayer;

mod api;
mod utils;
mod prisma;

fn router(client: Arc<prisma::PrismaClient>) -> axum::Router {
    let router = api::new().build().arced();

    axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route(
            "/rspc/:id",
            router
                .endpoint(move || api::Ctx {
                    client: Arc::clone(&client),
                })
                .axum(),
        )
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let client = Arc::new(prisma::new_client().await.unwrap());

    let addr = "[::]:9000".parse::<SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("{} listening on http://{}", env!("CARGO_CRATE_NAME"), addr);
    axum::Server::bind(&addr)
        .serve(router(client).into_make_service())
        .with_graceful_shutdown(utils::axum_shutdown_signal())
        .await
        .expect("Error with HTTP server!");
}
