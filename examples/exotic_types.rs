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
    time: Option<DateTime<Utc>>,
    s: &'static str
}

#[derive(Serialize, Type)]
struct GenericStruct<T> {
    x: T,
}

#[derive(Serialize, Type)]
enum SomeEnum<T> {
    Unit,
    Unnamed(Box<Vec<Option<T>>>),
    Named { n: Option<Vec<T>> },
}

// #[derive(Serialize, Type)]
// enum GenericEnum<T> {
//     X(T),
// }

// #[derive(Serialize, Type)]
// pub struct Demo {}

#[tokio::main]
async fn main() {
    let _r =
        <Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
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
            .query("genericStruct", |_, _: ()| GenericStruct::<String> {
                x: "Hello World".into(),
            })
            .query("enum", |_, _: ()| <SomeEnum<()>>::Unit)
            // .query("demo", |_, _: ()| {
            //     let mut x = BTreeMap::new();
            //     x.insert("a", Demo {});
            //     x
            // })
            // .query("genericEnum", |_, _: ()| {
            //     GenericEnum::<String>::X("Hello World".into())
            // })
            .query("uuid", |_, _: ()| {
                uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")
            })
            .query("chronoTimestamp", |_, _: ()| Utc::now())
            .query("exoticStruct", |_, _: ()| ExoticStruct {
                id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
                time: None,
                s: "Hello World",
            })
            .build();
}
