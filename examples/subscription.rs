use std::time::Duration;

use async_stream::stream;
use rspc::Router;
use serde_json::json;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let r = <Router>::new()
        .subscription("mySubscription", |t| {
            t(|_, _: ()| {
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
        })
        .build();

    // You usually don't use this method directly. An integration will handle this for you. Check out the Axum and Tauri integrations to see how to use them!
    // let stream = r
    //     .execute((), Operation::Subscription, "myQuery".into(), None)
    //     .await
    //     .unwrap();

    // TODO: Assertions
    // while let Some(msg) = stream.next().await {
    //     println!("Received: {:?}", msg);
    // }
}
