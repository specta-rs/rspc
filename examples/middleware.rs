use rspc::Router;
use serde_json::json;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .middleware(|_ctx, next| async move {
            println!("BEFORE");
            let v = next(42)?.await; // The value passed into next will the the context for all future operations.
            println!("AFTER");
            v
        })
        .query("version", |_ctx, _: ()| {
            println!("VERSION");
            env!("CARGO_PKG_VERSION")
        })
        // Middleware only apply to operations defined below them.
        .middleware(|_ctx, next| async move {
            println!("BEFORE ANOTHER");
            let v = next("todo")?.await;
            println!("AFTER ANOTHER");
            v
        })
        .query("another", |_ctx, _: ()| {
            println!("ANOTHER HANDLER");
            "Another Handler"
        })
        .build();

    println!(
        "{:#?}",
        router.exec_query((), "version", json!(null)).await.unwrap()
    );
    println!("");
    println!(
        "{:#?}",
        router.exec_query((), "another", json!(null)).await.unwrap()
    );
}
