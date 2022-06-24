use serde_json::json;
use trpc_rs::{Key, Router};

#[derive(Key, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum QueryKey {
    DemoQuery,
}

#[tokio::main]
async fn main() {
    let router = Router::<(), QueryKey>::new().query(QueryKey::DemoQuery, |_, _: ()| "Hello World");

    println!(
        "{:#?}",
        router
            .exec_query((), QueryKey::DemoQuery, json!(null))
            .await
            .unwrap()
    );
}
