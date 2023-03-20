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
    #[serde(default = "RequestId::null")] // Optional is not part of spec but copying tRPC
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
    Query {
        path: String,
        input: Option<Value>,
    },
    Mutation {
        path: String,
        input: Option<Value>,
    },
    Subscription {
        path: String,
        input: (RequestId, Option<Value>),
    },
    SubscriptionStop {
        input: RequestId,
    },
}

#[derive(Debug, Clone, Serialize)] // TODO: Add `specta::Type` when supported
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
