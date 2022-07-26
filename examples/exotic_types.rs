use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use chrono::prelude::*;
use rspc::{Config, Router, Type};
use serde::Serialize;
use serde_json::Value;
use uuid::{uuid, Uuid};

#[derive(Serialize, Type)]
struct ExoticStruct {
    id: Uuid,
}

#[tokio::main]
async fn main() {
    let _r = <Router>::new()
        .config(
            Config::new()
                .export_ts_bindings(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./ts")),
        )
        .query("hashmap", |_, _: ()| {
            let mut x = HashMap::new();
            x.insert("a", 1);
            x.insert("b", 2);
            x
        })
        .query("btreemap", |_, _: ()| {
            let mut x = BTreeMap::new();
            x.insert("a", 1);
            x.insert("b", 2);
            x
        })
        .query("serdeValue", |_, _: ()| Value::String("Hello World".into()))
        .query("uuid", |_, _: ()| {
            uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")
        })
        .query("chronoTimestamp", |_, _: ()| Utc::now())
        .query("exoticStruct", |_, _: ()| ExoticStruct {
            id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
        })
        .build();
}
