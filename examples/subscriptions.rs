use std::time::Duration;

use async_stream::stream;
use futures::StreamExt;
use rspc::{Config, Router};
use serde_json::json;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .config(Config::new().export_ts_bindings("./ts"))
        .subscription("pings", |_ctx, _args: ()| {
            stream! {
                println!("Client subscribed to 'pings'");
                for i in 0..5 {
                    println!("Sending ping {}", i);
                    yield "ping".to_string();
                    sleep(Duration::from_secs(1)).await;
                }
                println!("Client unsubscribed from 'pings'"); // TODO: This is not going to be run if client triggers shutdown cause we are doing a fixed loop
            }
        })
        .build();

    // In a real world application you would probably spawn this onto it's own tokio task.
    let mut stream = router
        .exec_subscription((), "pings", json!(null))
        .await
        .unwrap();
    while let Some(msg) = stream.next().await {
        println!("Transmitting: {:?}", msg);
    }
}
