use serde_json::json;

type Router = trpc_rs::Router<()>;

#[tokio::main]
async fn main() {
    let users_router = Router::new().query("list", |_, _: ()| vec![] as Vec<()>);

    let router = Router::new()
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .merge("users.", users_router);

    router.export("./ts").unwrap();

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
