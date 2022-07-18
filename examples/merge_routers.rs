/// This example show how to merge routers. It also demonstrates how they work with middleware context switching.
use rspc::{ActualMiddlewareResult, Config, MiddlewareResult, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    let users_router = Router::<i32>::new()
        .middleware(|_ctx, next| async {
            match next("todo")? {
                MiddlewareResult::Stream(stream) => Ok(stream.into_middleware_result()),
                result => {
                    let v = result.await?;
                    Ok(v.into_middleware_result())
                }
            }
        })
        .query("list", |_ctx, _: ()| vec![] as Vec<()>)
        .mutation("create", |_ctx, _: ()| todo!());

    let router = <Router>::new()
        .config(Config::new().export_ts_bindings("./ts"))
        .middleware(|_ctx, next| async {
            match next(42i32)? {
                MiddlewareResult::Stream(stream) => Ok(stream.into_middleware_result()),
                result => {
                    let v = result.await?;
                    Ok(v.into_middleware_result())
                }
            }
        })
        .query("version", |_ctx, _: ()| env!("CARGO_PKG_VERSION"))
        .merge("users.", users_router)
        .middleware(|ctx, next| async move {
            match next(ctx)? {
                MiddlewareResult::Stream(stream) => Ok(stream.into_middleware_result()),
                result => {
                    let v = result.await?;
                    Ok(v.into_middleware_result())
                }
            }
        })
        .query("another", |_ctx, _: ()| "Hello World")
        .build();

    println!(
        "{:#?}",
        router.exec_query((), "version", json!(null)).await.unwrap()
    );
    println!(
        "{:#?}",
        router
            .exec_query((), "users.list", json!(null))
            .await
            .unwrap()
    );
    println!(
        "{:#?}",
        router.exec_query((), "another", json!(null)).await.unwrap()
    );
}
