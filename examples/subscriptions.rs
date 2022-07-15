use rspc::Router;

#[tokio::main]
async fn main() {
    // THIS FEATURE IS A WORK IN PROGRESS. NOT READY FOR USE!!

    // let router = <Router>::new()
    //     .subscription("pings", |ctx| {
    //         println!("Client subscribed to 'pings'");
    //         tokio::spawn(async move {
    //             while let Some(msg) = ctx.next().await {
    //                 println!("Received: {:?}", msg);
    //                 ctx.emit("ping".to_string()).await;
    //             }
    //         });
    //     })
    //     .build();

    // println!(
    //     "{:#?}",
    //     router.exec_query((), "version", json!(null)).await.unwrap()
    // );
}
