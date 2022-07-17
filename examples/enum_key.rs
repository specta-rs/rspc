use rspc::{Config, Key, Router};
use serde_json::json;

#[derive(Key, Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum QueryKey {
    DemoQuery,
}

#[derive(Key, Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum MutationKey {
    DemoMutation,
}

#[tokio::main]
async fn main() {
    let router = Router::<(), (), QueryKey, MutationKey>::new()
        .config(Config::new().export_ts_bindings("./ts"))
        .query(QueryKey::DemoQuery, |_, _: ()| "Hello Query")
        .mutation(MutationKey::DemoMutation, |_, _: ()| "Hello Mutation")
        .build();

    println!(
        "{:#?}",
        router
            .exec_query((), QueryKey::DemoQuery, json!(null))
            .await
            .unwrap()
    );
    println!(
        "{:#?}",
        router
            .exec_mutation((), MutationKey::DemoMutation, json!(null))
            .await
            .unwrap()
    );
}
