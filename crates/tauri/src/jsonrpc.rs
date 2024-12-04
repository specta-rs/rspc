use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Null,
    Number(u32),
    String(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)] // TODO: Type on this
pub struct Request {
    pub jsonrpc: Option<String>, // This is required in the JsonRPC spec but I make it optional.
    pub id: RequestId,
    #[serde(flatten)]
    pub inner: RequestInner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ResponseInner {
    Event(Value),
    Response(Value),
    Error(JsonRPCError),
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRPCError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

// TODO: BREAK

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureKind {
    Query,
    Mutation,
    Subscription,
}

impl ProcedureKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            ProcedureKind::Query => "query",
            ProcedureKind::Mutation => "mutation",
            ProcedureKind::Subscription => "subscription",
        }
    }
}
