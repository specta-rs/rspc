use rspc_core2::{Format, TODOSerializer};
use serde_json::Value;

pub(crate) struct SerdeJsonFormat {}

impl Format for SerdeJsonFormat {
    type Result = serde_json::Value;
    type Serializer = SerdeJsonSerializer;

    fn serializer(&self) -> Self::Serializer {
        todo!()
    }
}

pub(crate) struct SerdeJsonSerializer(Option<Value>);

impl TODOSerializer for SerdeJsonSerializer {
    fn serialize_str(&mut self, s: &str) {
        self.0 = Some(Value::String(s.into()));
    }
}
