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
    time: Option<&'static (i32, i32)>,
    s: &'static str,
}

#[derive(Serialize, Type)]
struct GenericStruct<T> {
    x: T,
}

#[derive(Serialize, Type)]
enum SomeEnum {
    Unit,
    Unnamed(Box<Vec<Option<std::string::String>>>),
    Named { n: Option<Vec<i32>> },
}

#[derive(Serialize, Type)]
enum GenericEnum<T> {
    X(T),
}

#[derive(Serialize, Type)]
pub struct Demo();

#[tokio::main]
async fn main() {
    let _r =
        <Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            .query("hashmap", |t| {
                t(|_, _: ()| {
                    let mut x = HashMap::new();
                    x.insert("a", 1);
                    x.insert("b", 2);
                    x
                })
            })
            .query("btreemap", |t| {
                t(|_, _: ()| {
                    let mut x = BTreeMap::new();
                    x.insert("a", 1);
                    x.insert("b", 2);
                    x
                })
            })
            .query("serdeValue", |t| {
                t(|_, _: ()| Value::String("Hello World".into()))
            })
            .query("genericStruct", |t| {
                t(|_, _: ()| GenericStruct::<String> {
                    x: "Hello World".into(),
                })
            })
            .query("enum", |t| t(|_, _: ()| <SomeEnum>::Unit))
            .query("demo", |t| {
                t(|_, _: ()| {
                    let mut x = BTreeMap::new();
                    x.insert("a", Demo {});
                    x
                })
            })
            .query("genericEnum", |t| {
                t(|_, _: ()| GenericEnum::<String>::X("Hello World".into()))
            })
            .query("uuid", |t| {
                t(|_, _: ()| uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"))
            })
            .query("chronoTimestamp", |t| t(|_, _: ()| Utc::now()))
            .query("exoticStruct", |t| {
                t(|_, _: ()| ExoticStruct {
                    id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
                    time: None,
                    s: "Hello World",
                })
            })
            .build();
}
