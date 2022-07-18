use rspc::{ActualMiddlewareResult, MiddlewareResult, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .middleware(|_ctx, next| async move {
            println!("BEFORE");
            // The value passed into next will the the context for all future operations.
            match next(42)? {
                MiddlewareResult::Stream(stream) => Ok(stream.into_middleware_result()),
                result => {
                    let v = result.await?;
                    println!("AFTER");
                    Ok(v.into_middleware_result())
                }
            }
        })
        .query("version", |_ctx, _: ()| {
            println!("VERSION");
            env!("CARGO_PKG_VERSION")
        })
        // Middleware only apply to operations defined below them.
        .middleware(|_ctx, next| async move {
            println!("BEFORE ANOTHER");
            match next("todo")? {
                MiddlewareResult::Stream(stream) => Ok(stream.into_middleware_result()),
                result => {
                    let v = result.await?;
                    println!("AFTER ANOTHER");
                    Ok(v.into_middleware_result())
                }
            }
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
