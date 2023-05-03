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
        #[serde(default)] // TODO: Compatibility
        input: Option<Value>,
    },
    Mutation {
        path: String,
        #[serde(default)] // TODO: Compatibility
        input: Option<Value>,
    },
    Subscription {
        path: String,
        // The new system doesn't take an input but the old one does so this is design to make them compatible
        // TODO: Replace it with the new value
        #[serde(default)]
        input: NewOrOldInput,
    },
    // The new system doesn't take an input but the old one does so this is design to make them compatible
    // TODO: Remove value and `SubscriptionStop` struct in future
    SubscriptionStop(#[serde(default)] Option<SubscriptionStop>),
}

// TODO: Remove this in future
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
#[serde(untagged)]
pub enum NewOrOldInput {
    New(RequestId, Option<Value>),
    Old(Option<Value>),
}

impl Default for NewOrOldInput {
    fn default() -> Self {
        Self::Old(None)
    }
}

// TODO: Remove this in future
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
pub struct SubscriptionStop {
    pub input: RequestId,
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
