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

impl ToString for OperationKind {
    fn to_string(&self) -> String {
        match self {
            OperationKind::Query => "query",
            OperationKind::Mutation => "mutation",
            OperationKind::SubscriptionAdd => "subscriptionAdd",
            OperationKind::SubscriptionRemove => "subscriptionRemove",
        }
        .to_string()
    }
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
    Event {
        id: String,
        key: String,
        result: Value,
    },
    Response {
        id: Option<String>,
        result: Value,
    },
    Error {
        id: Option<String>,
        status_code: u16,
        message: String,
    },
    None,
}
