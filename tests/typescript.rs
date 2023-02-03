// use std::{
//     path::PathBuf,
//     process::{Command, Stdio},
// };

// use async_stream::stream;
// use rspc::{Config, Router, Type};
// use serde::{Deserialize, Serialize};

// #[derive(Type, Serialize, Deserialize)]
// pub enum Test {
//     Unit,
//     Unnamed(String),
// }

// #[derive(Type, Serialize, Deserialize)]
// pub struct Flatten {
//     #[serde(flatten)]
//     a: Test,
// }

// #[derive(Type, Deserialize)]
// pub struct PaginatedQueryArg {
//     cursor: String,
// }

// #[derive(Type, Deserialize)]
// pub struct PaginatedQueryArg2 {
//     cursor: String,
//     my_param: i32,
// }

// #[derive(Type, Serialize)]
// pub struct MyPaginatedData {
//     data: Vec<String>,
//     next_cursor: Option<String>,
// }

// fn export_rspc_types() {
//     let _r = <Router>::new()
//         .config(Config::new().export_ts_bindings(
//             PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./examples/astro/test/bindings.ts"),
//         ))
//         .query("noArgQuery", |t| t(|_, _: ()| "demo"))
//         .query("singleArgQuery", |t| t(|_, i: i32| i))
//         .query("flatteningQuery", |t| t(|_, a: Flatten| a))
//         .query("paginatedQueryOnlyCursor", |t| {
//             t(|_, _: PaginatedQueryArg| MyPaginatedData {
//                 data: vec!["a".to_string(), "b".to_string(), "c".to_string()],
//                 next_cursor: None,
//             })
//         })
//         .query("paginatedQueryCursorAndArg", |t| {
//             t(|_, _: PaginatedQueryArg2| MyPaginatedData {
//                 data: vec!["a".to_string(), "b".to_string(), "c".to_string()],
//                 next_cursor: None,
//             })
//         })
//         .mutation("noArgMutation", |t| t(|_, _: ()| "demo"))
//         .mutation("singleArgMutation", |t| t(|_, i: i32| i))
//         .subscription("noArgSubscription", |t| {
//             t(|_ctx, _args: ()| {
//                 stream! {
//                     yield "ping".to_string();
//                 }
//             })
//         })
//         .subscription("singleArgSubscription", |t| {
//             t(|_ctx, input: bool| {
//                 stream! {
//                     yield input;
//                 }
//             })
//         })
//         .build();
// }

// pub enum JSXMode {
//     React,
//     Solid,
// }

// fn tsc(file: &str, jsx_mode: JSXMode) {
//     let output = Command::new("tsc")
//         .arg("--esModuleInterop")
//         .arg("--strict")
//         .arg("--jsx")
//         .arg(match jsx_mode {
//             JSXMode::React => "react",
//             JSXMode::Solid => "preserve",
//         })
//         .arg("--lib")
//         .arg("es2015,dom")
//         .arg("--noEmit")
//         .arg(file)
//         .stdin(Stdio::null())
//         .stdout(Stdio::inherit())
//         .output()
//         .expect("failed to execute process");
//     assert!(output.status.success());
// }

// #[test]
// fn test_typescript_client() {
//     export_rspc_types();
//     // tsc("examples/astro/test/client.test.ts", JSXMode::React);
// }

// #[test]
// fn test_typescript_react() {
//     export_rspc_types();
//     // tsc("examples/astro/test/react.test.tsx", JSXMode::React);
// }

// #[test]
// fn test_typescript_sold() {
//     export_rspc_types();
//     // tsc("examples/astro/test/solid.test.tsx", JSXMode::Solid);
// }
