use async_stream::stream;
use rspc::{integrations::httpz::Request, Config};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

struct Ctx {
    x_demo_header: Option<String>,
}

#[tokio::main]
async fn main() {
    // let router =
    //     rspc::Router::<Ctx>::new()
    //         .config(Config::new().export_ts_bindings(
    //             PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bindings.ts"),
    //         ))
    //         .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
    //         .query("X-Demo-Header", |t| {
    //             t(|ctx, _: ()| ctx.x_demo_header.unwrap_or_else(|| "No header".to_string()))
    //         })
    //         .query("echo", |t| t(|_, v: String| v))
    //         .query("error", |t| {
    //             t(|_, _: ()| {
    //                 Err(rspc::Error::new(
    //                     rspc::ErrorCode::InternalServerError,
    //                     "Something went wrong".into(),
    //                 )) as Result<String, rspc::Error>
    //             })
    //         })
    //         .mutation("error", |t| {
    //             t(|_, _: ()| {
    //                 Err(rspc::Error::new(
    //                     rspc::ErrorCode::InternalServerError,
    //                     "Something went wrong".into(),
    //                 )) as Result<String, rspc::Error>
    //             })
    //         })
    //         .query("transformMe", |t| t(|_, _: ()| "Hello, world!".to_string()))
    //         .mutation("sendMsg", |t| {
    //             t(|_, v: String| {
    //                 println!("Client said '{}'", v);
    //                 v
    //             })
    //         })
    //         .subscription("pings", |t| {
    //             t(|_ctx, _args: ()| {
    //                 stream! {
    //                     println!("Client subscribed to 'pings'");
    //                     for i in 0..5 {
    //                         println!("Sending ping {}", i);
    //                         yield "ping".to_string();
    //                         sleep(Duration::from_secs(1)).await;
    //                     }
    //                 }
    //             })
    //         })
    //         // TODO: Results being returned from subscriptions
    //         // .subscription("errorPings", |t| t(|_ctx, _args: ()| {
    //         //     stream! {
    //         //         for i in 0..5 {
    //         //             yield Ok("ping".to_string());
    //         //             sleep(Duration::from_secs(1)).await;
    //         //         }
    //         //         yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
    //         //     }
    //         // }))
    //         .build()
    //         .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    // router
    //     .endpoint(|req: Request| {
    //         println!("Client requested operation '{}'", req.uri().path());
    //         Ctx {
    //             x_demo_header: req
    //                 .headers()
    //                 .get("X-Demo-Header")
    //                 .map(|v| v.to_str().unwrap().to_string()),
    //         }
    //     })
    //     .vercel()
    //     .await
    //     .unwrap();
}
