// TODO: Unit testing with Tokio loom

// use streamunordered::StreamUnordered;
//
// construct a basic router for testing the executor
fn router() {
    todo!();
}

// Replicate a websocket-style setup where we have a `Stream + Sink` and wanna restrict each client to a single task.
#[tokio::test]
async fn test_executor_websocket_like() {
    // let executor = todo!();

    // tokio::spawn(async move {
    //     // let futs = StreamUnordered<Ready<()>>::new():
    //     // let conn = Connection::new();
    //     tokio::select! {
    //         // _ = futs => {
    //             // TODO: tx.send();
    //         // }
    //         // _ = rx.recv() => {
    //             // TODO: executor.execute(&mut conn, ctx, req);
    //             // TODO: either tx.send() or futs.push();
    //         // }
    //     }
    //     // let
    // });
}

// // Just doing regular in-memory query/mutations & subscriptions with a little code as possible
// #[tokio::test]
// async fn test_executor_minimal() {
//     todo!();
// }

// TODO: Unit test batching
