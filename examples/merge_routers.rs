/// This example show how to merge routers. It also demonstrates how they work with middleware context switching.
use rspc::Router;
use serde_json::json;

#[tokio::main]
async fn main() {
    let users_router = Router::<i32>::new()
        .middleware(|_ctx, next| async { next("todo")?.await })
        .query("list", |_ctx, _: ()| vec![] as Vec<()>);

    let router = <Router>::new()
        .middleware(|_ctx, next| async { next(42)?.await })
        .query("version", |_ctx, _: ()| env!("CARGO_PKG_VERSION"))
        .merge("users.", users_router)
        .middleware(|ctx, next| async move { next(ctx)?.await })
        .query("another", |_ctx, _: ()| "Hello World")
        .build();

    // router.export("./ts").unwrap(); // TODO

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
}
