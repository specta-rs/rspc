use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::routing::get;
use rspc::{integrations::httpz::Request, ErrorCode, ExportConfig, Rspc};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct Ctx {
    x_demo_header: Option<String>,
}

const R: Rspc<Ctx> = Rspc::new();

#[tokio::main]
async fn main() {
    let router =
        R.router()
            .procedure("version", R.query(|_, _: ()| env!("CARGO_PKG_VERSION")))
            .procedure(
                "X-Demo-Header",
                R.query(|ctx, _: ()| {
                    ctx.x_demo_header
                        .unwrap_or_else(|| "No header".to_string())
                }),
            )
            .procedure("echo", R.query(|_, v: String| v))
            .procedure(
                "error",
                R.query(|_, _: ()| {
                    Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        "Something went wrong".into(),
                    )) as Result<String, rspc::Error>
                }),
            )
            .procedure(
                "error",
                R.mutation(|_, _: ()| {
                    Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        "Something went wrong".into(),
                    )) as Result<String, rspc::Error>
                }),
            )
            .procedure(
                "transformMe",
                R.query(|_, _: ()| "Hello, world!".to_string()),
            )
            .procedure(
                "sendMsg",
                R.mutation(|_, v: String| {
                    println!("Client said '{}'", v);
                    v
                }),
            )
            .procedure(
                "pings",
                R.subscription(|_, _: ()| {
                    println!("Client subscribed to 'pings'");
                    stream! {
                        yield "start".to_string();

                        for i in 0..5 {
                            println!("Sending ping {}", i);
                            yield i.to_string();
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }),
            )
            .procedure("errorPings", R.subscription(|_ctx, _args: ()| {
                stream! {
                    for _ in 0..5 {
                        yield Ok("ping".to_string());
                        sleep(Duration::from_secs(1)).await;
                    }
                    yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
                }
            }))
            .build()
            .unwrap()
            .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
        ))
        .unwrap();

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
                    Ctx {
                        x_demo_header: req
                            .headers()
                            .get("X-Demo-Header")
                            .map(|v| v.to_str().unwrap().to_string()),
                    }
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
