use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::routing::get;
use rspc::{integrations::httpz::Request, Config};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

struct Ctx {}

#[tokio::main]
async fn main() {
    let router =
        rspc::Router::<Ctx>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
            ))
            .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
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
            .build()
            .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            router
                .clone()
                .endpoint(|req: Request| {
                    println!("Client requested operation '{}'", req.uri().path());
                    Ctx {}
                })
                .axum(),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
