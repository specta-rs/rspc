use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum Request {
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
        id: u32,
        input: Option<Value>,
    },
    SubscriptionStop {
        id: u32,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum Response {
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
