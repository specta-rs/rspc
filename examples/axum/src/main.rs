use axum::{
    body::Body,
    extract::Multipart,
    http::{header, request::Parts, HeaderMap},
    routing::{get, post},
};
use example_core::{mount, Ctx};
use futures::{Stream, StreamExt};
use rspc::{DynOutput, ProcedureError, ProcedureStream, ProcedureStreamMap, Procedures};
use rspc_invalidation::Invalidator;
use serde_json::Value;
use std::{
    convert::Infallible,
    future::poll_fn,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};
use streamunordered::{StreamUnordered, StreamYield};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = mount();
    let (procedures, types) = router.build().unwrap();

    rspc::Typescript::default()
        // .formatter(specta_typescript::formatter::prettier)
        .header("// My custom header")
        // .enable_source_maps() // TODO: Fix this
        .export_to(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
            &types,
        )
        .unwrap();

    // Be aware this is very experimental and doesn't support many types yet.
    // rspc::Rust::default()
    //     // .header("// My custom header")
    //     .export_to(
    //         PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../client/src/bindings.rs"),
    //         &types,
    //     )
    //     .unwrap();

    // let procedures = rspc_devtools::mount(procedures, &types); // TODO

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "rspc 🤝 Axum!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(procedures, |parts: Parts| {
                println!("Client requested operation '{}'", parts.uri.path());
                Ctx {}
            }),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
