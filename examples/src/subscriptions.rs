use std::time::Duration;

use async_stream::stream;
use rspc::RouterBuilder;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::time::sleep;

use crate::R;

#[derive(Serialize, Type, Debug, Error)]
#[error("{}", .0)]
struct Error(String);

// We merge this router into the main router in `main.rs`.
// This router shows how to do subscriptions.
pub fn mount() -> RouterBuilder<()> {
    R.router()
        .procedure(
            "pings",
            R.subscription(|_ctx, _args: ()| {
                stream! {
                    println!("Client subscribed to 'pings'");
                    for i in 0..5 {
                        println!("Sending ping {}", i);
                        yield Ok("ping".to_string());
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }),
        )
        .procedure(
            "asyncPings",
            R.subscription(|_ctx, _args: ()| async move {
                stream! {
                    for _ in 0..5 {
                        yield Ok("ping".to_string());
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }),
        )
        .procedure(
            "errorPings",
            R.error::<Error>().subscription(|_ctx, _args: ()| {
                stream! {
                    for _ in 0..5 {
                        yield Ok("ping".to_string());
                        sleep(Duration::from_secs(1)).await;
                    }
                    yield Err(Error("Something went wrong".to_string()));
                }
            }),
        )
}
