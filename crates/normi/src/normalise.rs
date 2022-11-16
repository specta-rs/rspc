use crate::{Object, RefMap};
use serde_json::Value;

pub fn normalise(value: impl Object) -> Result<Value, serde_json::Error> {
    let mut refs = RefMap::default();
    Ok(Value::Object({
        let mut map = serde_json::Map::new();
        map.insert("$data".to_string(), value.normalize(&mut refs).unwrap());
        map.insert(
            "$refs".to_string(),
            Value::Array(
                refs.into_iter()
                    .map(|(k, v)| {
                        match v {
                            Value::Object(mut map) => {
                                map.insert("$id".to_string(), k.id.into());
                                map.insert("$ty".to_string(), k.ty.into());
                                Value::Object(map)
                            }
                            _ => panic!("Expected object"), // TODO: Is this something I need to handle?
                        }
                    })
                    .collect(),
            ),
        );
        map
    }))
}
