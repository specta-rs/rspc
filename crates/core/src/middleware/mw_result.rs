use serde_json::Value;

use crate::error::ExecError;

/// TODO
pub trait IntoMiddlewareResult {
    // TODO: Support streams and bytes
    fn into_result(self) -> Result<Value, ExecError>;
}

impl IntoMiddlewareResult for () {
    fn into_result(self) -> Result<Value, ExecError> {
        Ok(Value::Null)
    }
}

impl IntoMiddlewareResult for Value {
    fn into_result(self) -> Result<Value, ExecError> {
        Ok(self)
    }
}
