use std::time::Duration;

use async_stream::stream;
use rspc::{Router, RouterBuilder};
use tokio::time::sleep;

// We merge this router into the main router in `main.rs`.
// This router shows how to do subscriptions.
pub fn mount() -> RouterBuilder {
    Router::new().subscription("pings", |t| {
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
}
