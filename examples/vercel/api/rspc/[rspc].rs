use async_stream::stream;
use rspc::{ExportConfig, Rspc};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

#[derive(Clone)]
struct Ctx {
    x_demo_header: Option<String>,
}

#[derive(thiserror::Error, serde::Serialize, specta::Type, Debug)]
#[error("{0}")]
struct Error(&'static str);

const R: Rspc<Ctx, Error> = Rspc::new();

#[tokio::main]
async fn main() {
    let router = R
        .router()
        .procedure("version", R.query(|_, _: ()| Ok(env!("CARGO_PKG_VERSION"))))
        .procedure(
            "X-Demo-Header",
            R.query(|ctx, _: ()| Ok(ctx.x_demo_header.unwrap_or_else(|| "No header".to_string()))),
        )
        .procedure("echo", R.query(|_, v: String| Ok(v)))
        .procedure(
            "error",
            R.query(|_, _: ()| Err(Error("Something went wrong")) as Result<String, _>),
        )
        .procedure(
            "error",
            R.mutation(|_, _: ()| Err(Error("Something went wrong")) as Result<String, _>),
        )
        .procedure(
            "transformMe",
            R.query(|_, _: ()| Ok("Hello, world!".to_string())),
        )
        .procedure(
            "sendMsg",
            R.mutation(|_, v: String| {
                println!("Client said '{}'", v);
                Ok(v)
            }),
        )
        .procedure(
            "pings",
            R.subscription(|_ctx, _args: ()| {
                stream! {
                    println!("Client subscribed to 'pings'");
                    for i in 0..5 {
                        println!("Sending ping {}", i);
                        yield "ping".to_string();
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }),
        )
        .build()
        .unwrap()
        .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bindings.ts"),
        ))
        .unwrap();

    rspc_httpz::endpoint(router, |req: rspc_httpz::Request| {
        println!("Client requested operation '{}'", req.uri().path());
        Ctx {
            x_demo_header: req
                .headers()
                .get("X-Demo-Header")
                .map(|v| v.to_str().unwrap().to_string()),
        }
    })
    .vercel()
    .await
    .unwrap();
}
