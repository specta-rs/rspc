//! This example shows using `rspc_legacy` directly.
//! This is not intended for permanent use, but instead it is designed to allow an incremental migration from rspc 0.3.0.

use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::{http::request::Parts, routing::get};
use rspc_legacy::{Error, ErrorCode, Router, RouterBuilder};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

pub(crate) struct Ctx {}

fn mount() -> RouterBuilder<Ctx> {
    Router::<Ctx>::new()
        .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .query("echo", |t| t(|_, v: String| v))
        .query("error", |t| {
            t(|_, _: ()| {
                Err(Error::new(
                    ErrorCode::InternalServerError,
                    "Something went wrong".into(),
                )) as Result<String, Error>
            })
        })
        .query("transformMe", |t| t(|_, _: ()| "Hello, world!".to_string()))
        .mutation("sendMsg", |t| {
            t(|_, v: String| {
                println!("Client said '{}'", v);
                v
            })
        })
        .subscription("pings", |t| {
            t(|_ctx, _args: ()| {
                stream! {
                    println!("Client subscribed to 'pings'");
                    for i in 0..5 {
                        println!("Sending ping {}", i);
                        yield "ping".to_string();
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            })
        })
}

#[tokio::main]
async fn main() {
    let (procedures, types) = rspc::Router2::from(mount().build()).build().unwrap();

    rspc::Typescript::default()
        .export_to(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
            &types,
        )
        .unwrap();

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello from rspc legacy!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(procedures, |parts: Parts| {
                println!("Client requested operation '{}'", parts.uri.path());
                Ctx {}
            }),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
