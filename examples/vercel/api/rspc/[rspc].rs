use async_stream::stream;
use rspc::{integrations::httpz::Request, ExportConfig, Rspc};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

#[derive(Clone)]
struct Ctx {
    x_demo_header: Option<String>,
}

const R: Rspc<Ctx> = Rspc::new();

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

    router
        .endpoint(|req: Request| {
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
