use crate::api::Ctx;
use axum::routing::get;
use std::net::SocketAddr;

mod api;
mod utils;

fn router() -> axum::Router {
    let router = api::new().build().arced();

    axum::Router::new()
        .route("/", get(|| async { "Welcome to your new rspc app!" }))
        .route("/health", get(|| async { "Ok!" }))
        .route("/rspc/:id", router.endpoint(|| Ctx {}).axum())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let addr = "[::]:9000".parse::<SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("{} listening on http://{}", env!("CARGO_CRATE_NAME"), addr);
    axum::Server::bind(&addr)
        .serve(router().into_make_service())
        .with_graceful_shutdown(utils::axum_shutdown_signal())
        .await
        .expect("Error with HTTP server!");
}
