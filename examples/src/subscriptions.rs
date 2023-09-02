use std::time::Duration;

use async_stream::stream;
use rspc::{ErrorCode, Router};
use tokio::time::sleep;

use crate::R;

// We merge this router into the main router in `main.rs`.
// This router shows how to do subscriptions.
pub fn mount() -> Router<()> {
    R.router().procedure(
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
    // .procedure(
    //     "asyncPings",
    //     R.subscription(|_ctx, _args: ()| async move {
    //         stream! {
    //             for _ in 0..5 {
    //                 yield Ok("ping".to_string());
    //                 sleep(Duration::from_secs(1)).await;
    //             }
    //         }
    //     }),
    // )
    // .procedure("errorPings", R.subscription(|_ctx, _args: ()| {
    //     stream! {
    //         for _ in 0..5 {
    //             yield Ok("ping".to_string());
    //             sleep(Duration::from_secs(1)).await;
    //         }
    //         yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
    //     }
    // }))
}
