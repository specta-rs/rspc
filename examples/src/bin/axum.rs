#![allow(clippy::unused_unit)]

use std::path::PathBuf;

use example::{basic, selection, subscriptions};

use axum::{http::request::Parts, routing::get};
use rspc::{Config, Router};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let r1 = Router::<i32>::new().query("demo", |t| t(|_, _: ()| "Merging Routers!"));

    let router =
        <rspc::Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            // Basic query
            .query("version", |t| {
                t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") })
            })
            .merge("basic.", basic::mount())
            .merge("subscriptions.", subscriptions::mount())
            .merge("selection.", selection::mount())
            // This middleware changes the TCtx (context type) from `()` to `i32`. All routers being merge under need to take `i32` as their context type.
            .middleware(|mw| mw.middleware(|ctx| async move { Ok(ctx.with_ctx(42i32)) }))
            .merge("r1.", r1)
            .build()
            .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    let app = axum::Router::new()
        .with_state(())
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest(
            "/rspc",
            rspc_axum::endpoint(router, |parts: Parts| {
                println!("Client requested operation '{}'", parts.uri.path());

                ()
            }),
        )
        // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
