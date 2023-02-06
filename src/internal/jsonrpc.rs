use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use super::jsonrpc_exec::*;

/// TODO
///
/// @internal
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(specta::Type))]
#[serde(untagged)]
pub enum RequestId {
    Null,
    Number(u32),
    String(String),
}

impl RequestId {
    fn null() -> Self {
        Self::Null
    }
}

/// TODO
///
/// @internal
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
pub struct Request {
    pub jsonrpc: Option<String>, // This is required in the JsonRPC spec but I make it optional.
    #[serde(default = "RequestId::null")] // TODO: Optional also not part of spec but copying tRPC
    pub id: RequestId,
    #[serde(flatten)]
    pub inner: RequestInner,
}

/// TODO
///
/// @internal
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum RequestInner {
    Query { path: String, input: Option<Value> },
    Mutation { path: String, input: Option<Value> },
    Subscription { path: String, input: Option<Value> },
    SubscriptionStop,
}

/// TODO
///
/// @internal
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: RequestId,
    pub result: ResponseInner,
}

/// TODO
///
/// @internal
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ResponseInner {
    Event(Value),
    Response(Value),
    Error(JsonRPCError),
}

/// TODO
///
/// @internal
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
pub struct JsonRPCError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

// #[cfg(test)]
// mod tests {
//     use std::{fs::File, io::Write, path::PathBuf};

//     use super::*;

//     #[test]
//     fn export_internal_bindings() {
//         // let mut file = File::create(
//         //     PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./packages/client/src/types.ts"),
//         // )
//         // .unwrap();
//         // file.write_all(
//         //     b"// Do not modify this file. It was generated from the Rust types by running ``.\n\n",
//         // )
//         // .unwrap();
//         // // TODO: Add an API into Specta which allows exporting a type and all types it depends on.
//         // file.write_all(format!("{}\n\n", specta::ts_export::<RequestId>().unwrap()).as_bytes())
//         //     .unwrap();
//         // file.write_all(format!("{}\n\n", specta::ts_export::<Request>().unwrap()).as_bytes())
//         //     .unwrap();
//     }

//     #[test]
//     fn test_request_id() {
//         // println!(
//         //     "{}",
//         //     serde_json::to_string(&Request {
//         //         jsonrpc: None,
//         //         id: RequestId::Null,
//         //         inner: RequestInner::Query {
//         //             path: "test".into(),
//         //             input: None,
//         //         },
//         //     })
//         //     .unwrap()
//         // );
//         todo!();

//         // TODO: Test serde

//         // TODO: Test specta
//     }

//     #[test]
//     fn test_jsonrpc_request() {
//         todo!();
//     }

//     #[test]
//     fn test_jsonrpc_response() {
//         todo!();
//     }
// }
