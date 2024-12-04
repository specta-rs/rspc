use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::{http::request::Parts, routing::get};
use rspc::{Config, Router2};
use serde::Serialize;
use specta::Type;
use specta_typescript::Typescript;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

struct Ctx {}

#[derive(Serialize, Type)]
pub struct MyCustomType(String);

#[derive(Type, Serialize)]
#[serde(tag = "type")]
#[specta(export = false)]
pub enum DeserializationError {
    // Is not a map-type so invalid.
    A(String),
}

// http://[::]:4000/rspc/version
// http://[::]:4000/legacy/version

// http://[::]:4000/rspc/nested.hello
// http://[::]:4000/legacy/nested.hello

// http://[::]:4000/rspc/error
// http://[::]:4000/legacy/error

// http://[::]:4000/rspc/echo
// http://[::]:4000/legacy/echo

// http://[::]:4000/rspc/echo?input=42
// http://[::]:4000/legacy/echo?input=42

fn mount() -> rspc::Router<Ctx> {
    let inner = rspc::Router::<Ctx>::new().query("hello", |t| t(|_, _: ()| "Hello World!"));

    let router = rspc::Router::<Ctx>::new()
        .merge("nested.", inner)
        .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .mutation("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .query("echo", |t| t(|_, v: String| v))
        .query("error", |t| {
            t(|_, _: ()| {
                Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    "Something went wrong".into(),
                )) as Result<String, rspc::Error>
            })
        })
        .query("transformMe", |t| t(|_, _: ()| "Hello, world!".to_string()))
        .mutation("sendMsg", |t| {
            t(|_, v: String| {
                println!("Client said '{}'", v);
                v
            })
        })
        .mutation("anotherOne", |t| t(|_, v: String| Ok(MyCustomType(v))))
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
        // TODO: Results being returned from subscriptions
        // .subscription("errorPings", |t| t(|_ctx, _args: ()| {
        //     stream! {
        //         for i in 0..5 {
        //             yield Ok("ping".to_string());
        //             sleep(Duration::from_secs(1)).await;
        //         }
        //         yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
        //     }
        // }))
        .build();

    router
}

#[tokio::main]
async fn main() {
    let (routes, types) = Router2::from(mount()).build().unwrap();

    types
        .export_to(
            Typescript::default(),
            // .formatter(specta_typescript::formatter::prettier),
            // .header("// My custom header\n")
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
        )
        .unwrap();

    // TODO: Export the legacy bindings from a new router

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(routes, |parts: Parts| {
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
