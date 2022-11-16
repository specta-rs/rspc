use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Number};

/// Fork of [serde_json::Value] which allows for `Ord` and `PartialOrd` so it can be put as the key for a `BTreeSet`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl From<Value> for serde_json::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(b),
            Value::Number(n) => Self::Number(n),
            Value::String(s) => Self::String(s),
            Value::Array(a) => Self::Array(a.into_iter().map(Self::from).collect()),
            Value::Object(o) => Self::Object(
                o.into_iter()
                    .map(|(k, v)| (k, Self::from(v)))
                    .collect::<Map<_, _>>(),
            ),
        }
    }
}

impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Value {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(n),
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(a) => Value::Array(a.into_iter().map(Self::from).collect()),
            serde_json::Value::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| (k, Self::from(v)))
                    .collect::<BTreeMap<_, _>>(),
            ),
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(a) => a.serialize(serializer),
            Value::Object(o) => o.serialize(serializer),
        }
    }
}
