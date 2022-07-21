use std::time::Duration;

use async_stream::stream;
use axum::{extract::Path, routing::get};
use rspc::Config;
use serde::Deserialize;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

struct Ctx {
    library_id: String,
}

// This could be the same struct as `Ctx` here but in a real world application you will always wanna separate them.
#[derive(Deserialize)]
struct RequestCtx {
    library_id: String,
}

#[tokio::main]
async fn main() {
    let router = rspc::Router::<Ctx>::new()
        .config(Config::new().export_ts_bindings("./examples/ts"))
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .query("getUser", |_, v: i32| v)
        .query("getCurrentLibrary", |ctx, _: ()| ctx.library_id.clone())
        .mutation("sayHi", |_, v: String| println!("{}", v))
        .subscription("pings", |_ctx, _args: ()| {
            stream! {
                println!("Client subscribed to 'pings'");
                for i in 0..5 {
                    println!("Sending ping {}", i);
                    yield "ping".to_string();
                    sleep(Duration::from_secs(1)).await;
                }
                println!("Client unsubscribed from 'pings'"); // TODO: This is not going to be run if client triggers shutdown cause we are doing a fixed loop
            }
        })
        .build()
        .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route(
            "/rspc/:id",
            router.clone().axum_handler(|Path(path): Path<String>| {
                println!("Client requested operation '{}'", path);
                Ctx {
                    library_id: "todo".into(),
                }
            }),
        )
        .route(
            "/rspcws",
            router.axum_ws_handler(|| Ctx {
                library_id: "todo".into(),
            }),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
