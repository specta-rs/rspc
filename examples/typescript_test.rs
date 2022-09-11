//! Running this file will create the bindings for the Typescript unit tests on the frontend.

use async_stream::stream;
use rspc::{Config, Router};

#[tokio::main]
async fn main() {
    let _r = <Router>::new()
        .config(Config::new().export_ts_bindings("./packages/example/bindings.ts"))
        .query("noArgQuery", |t| t(|_, _: ()| "demo"))
        .query("singleArgQuery", |t| t(|_, i: i32| i))
        .mutation("noArgMutation", |t| t(|_, _: ()| "demo"))
        .mutation("singleArgMutation", |t| t(|_, i: i32| i))
        .subscription("noArgSubscription", |t| {
            t(|_ctx, _args: ()| {
                stream! {
                    yield "ping".to_string();
                }
            })
        })
        .subscription("singleArgSubscription", |t| {
            t(|_ctx, arg: bool| {
                stream! {
                    yield arg;
                }
            })
        })
        .build();
}
