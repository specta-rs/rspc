use std::sync::Arc;

use axum::{extract::Path, routing::get};
use rspc::SerdeTypeMarker;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = rspc::Router::<()>::new()
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .query("getUser", |_, v: i32| v)
        .mutation("sayHi", |_, v: String| println!("{}", v))
        // TODO: Make it so you don't have to specify all these generics
        .subscription::<&'static str, (), SerdeTypeMarker, String>("pings", |_ctx| {
            println!("Client subscribed to 'pings'");
            // TODO: Implement system for listening and sending messages to client
        });
    let router = Arc::new(router.build());

    router.export("./examples/solid/src/ts").unwrap();

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route(
            "/rspc/:id",
            router.clone().axum_handler(|Path(_key): Path<String>| ()),
        )
        .route("/rspcws", router.axum_ws_handler(|| ()))
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!(
        "listening on http://{}/rspc/version?batch=1&input=%7B%7D",
        addr
    );
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
