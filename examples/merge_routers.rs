use std::path::PathBuf;

use rspc::{Config, OperationKey, OperationKind, Router, StreamOrValue};
use serde_json::json;

#[tokio::main]
async fn main() {
    let r1 = Router::<i32>::new().query("demo", |t| t(|_, _: ()| "Merging Routers!"));

    let r =
        <Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            .middleware(|ctx| async move { ctx.next(42).await })
            .query("version", |t| t(|_, _: ()| "0.1.0"))
            .merge("r1.", r1)
            .build();

    // You usually don't use this method directly. An integration will handle this for you. Check out the Axum and Tauri integrations to see how to use them!
    match r
        .exec(
            (),
            OperationKind::Query,
            OperationKey("version".into(), None),
        )
        .await
        .unwrap()
    {
        StreamOrValue::Stream(_) => unreachable!(),
        StreamOrValue::Value(v) => {
            println!("{:?}", v);
            assert_eq!(v, json!("0.1.0"));
        }
    }

    match r
        .exec(
            (),
            OperationKind::Query,
            OperationKey("r1.demo".into(), None),
        )
        .await
        .unwrap()
    {
        StreamOrValue::Stream(_) => unreachable!(),
        StreamOrValue::Value(v) => {
            println!("{:?}", v);
            assert_eq!(v, json!("Merging Routers!"));
        }
    }
}
