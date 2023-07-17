// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use async_stream::stream;
use rspc::{ErrorCode, Rspc};
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
    .procedure(
        "version",
        R
            .with(|mw, ctx| async move {
                mw.next(ctx).map(|resp| async move {
                    println!("Client requested version '{}'", resp);
                    resp
                })
            })
            .with(|mw, ctx| async move { mw.next(ctx) })
            .query(|_, _: ()| env!("CARGO_PKG_VERSION")),
    )
    .procedure(
        "X-Demo-Header",
        R.query(|ctx, _: ()| ctx.x_demo_header.unwrap_or_else(|| "No header".to_string())),
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

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router, |_| Ctx {
            x_demo_header: None,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
