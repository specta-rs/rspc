use std::any::Any;

use serde_json::Value;

/// TODO
pub enum ConcreteArg {
    Value(Value),
    Unknown(Box<dyn Any + Send + 'static>), // TODO: Remove this variant. It's more less overhead but it doesn't worth with middleware arguments.
}
