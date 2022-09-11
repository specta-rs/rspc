use std::path::PathBuf;

use rspc::{Config, OperationKey, OperationKind, Router, StreamOrValue};
use serde_json::json;

#[tokio::main]
async fn main() {
    let r =
        <Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            .query("myQuery", |t| t(|_, _: ()| "My Query Result!"))
            .query("anotherQuery", |t| {
                t.resolver(|_, _: ()| "My Query Result!")
            })
            .mutation("myMutation", |t| t(|_ctx, arg: i32| arg))
            .build();

    // You can also export the bindings yourself
    // r.export_ts("./bindings.ts").unwrap();

    // You usually don't use this method directly. An integration will handle this for you. Check out the Axum and Tauri integrations to see how to use them!
    match r
        .exec(
            (),
            OperationKind::Query,
            OperationKey("myQuery".into(), None),
        )
        .await
        .unwrap()
    {
        StreamOrValue::Stream(_) => unreachable!(),
        StreamOrValue::Value(v) => {
            println!("{:?}", v);
            assert_eq!(v, json!("My Query Result!"));
        }
    }

    match r
        .exec(
            (),
            OperationKind::Mutation,
            OperationKey("myMutation".into(), Some(json!(5))),
        )
        .await
        .unwrap()
    {
        StreamOrValue::Stream(_) => unreachable!(),
        StreamOrValue::Value(v) => {
            println!("{:?}", v);
            assert_eq!(v, json!(5));
        }
    }
}
