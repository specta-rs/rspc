use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type KindAndKey = (OperationKind, OperationKey);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationKey(
    pub String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub Option<Value>,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationKind {
    Query,
    Mutation,
    SubscriptionAdd,
    SubscriptionRemove,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Option<String>,
    pub operation: OperationKind,
    pub key: OperationKey,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Response {
    Event(EventResult),
    Response(ResponseResult),
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventResult {
    pub key: String,
    pub result: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResponseResult {
    Success { id: Option<String>, result: Value },
    Error, // TODO: Make events work
}
