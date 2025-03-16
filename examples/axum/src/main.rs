use axum::{
    body::Body,
    extract::Multipart,
    http::{header, request::Parts, HeaderMap},
    routing::{get, post},
};
use example_core::{mount, BaseProcedure, Ctx};
use futures::{Stream, StreamExt};
use rspc::{
    DynOutput, Procedure, ProcedureError, ProcedureStream, ProcedureStreamMap, Procedures, Router,
};
use rspc_invalidation::Invalidator;
use serde_json::Value;
use std::{
    convert::Infallible,
    future::poll_fn,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use streamunordered::{StreamUnordered, StreamYield};
use tower_cookies::Cookies;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = mount()
        .procedure(
            "flush",
            <BaseProcedure>::builder().query(async |_, _: ()| {
                println!("1 not flushed yet");
                tokio::time::sleep(Duration::from_secs(1)).await;
                rspc_axum::flush();
                println!("flushed 1");
                tokio::time::sleep(Duration::from_secs(3)).await;
                Ok("flush 1")
            }),
        )
        .procedure(
            "flush2",
            <BaseProcedure>::builder().query(async |_, _: ()| {
                println!("2 not flushed yet");
                tokio::time::sleep(Duration::from_secs(2)).await;
                rspc_axum::flush();
                println!("flushed 2");
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok("flush 2")
            }),
        );

    // cookie flushing test
    // let router = Router::new().procedure(
    //     "sets-cookie",
    //     rspc::Procedure::builder::<example_core::Error>().query(
    //         async |cookies: tower_cookies::Cookies, _: ()| {
    //             cookies.add(tower_cookies::Cookie::new("authorization", "Bearer ABCD"));
    //             rspc_axum::flush();

    //             Ok("should have cookie set")
    //         },
    //     ),
    // );

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
            rspc_axum::Endpoint::new(procedures, |parts: Parts| {
                println!("Client requested operation '{}'", parts.uri.path());
                Ctx {}
            })
            // rspc_axum::Endpoint::new(procedures, |cookies: Cookies| cookies)
            // .manual_stream_flushing()
            .build(),
        )
        .layer(tower_cookies::CookieManagerLayer::new())
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
