use std::any::Any;

use serde_json::Value;

/// TODO
pub enum ConcreteArg {
    Value(Value),
    Unknown(Box<dyn Any + Send + Sync + 'static>),
}
