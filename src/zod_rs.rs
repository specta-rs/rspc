use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub trait ZData {
    type Output: Into<Value>;

    fn parse(&self, data: Value) -> Result<Self::Output, ()>;
}

pub struct ZString;

impl ZData for ZString {
    type Output = String;

    fn parse(&self, data: Value) -> Result<Self::Output, ()> {
        Ok(data.as_str().ok_or(())?.to_string())
    }
}

pub struct ZNumber;

impl ZData for ZNumber {
    type Output = i64; // TODO: Support all number formats using generic?

    fn parse(&self, data: Value) -> Result<Self::Output, ()> {
        Ok(data.as_i64().ok_or(())?)
    }
}

pub struct ZObject(BTreeMap<&'static str, Box<dyn Fn(Value) -> Result<Value, ()>>>);

impl ZObject {
    // TODO: Make safe variants which returns error and doesn't panic
    pub fn field<TData: ZData + 'static>(mut self, key: &'static str, value: TData) -> Self {
        if self.0.contains_key(&key) {
            panic!("zod-rs error: object already has key '{}'", key);
        }

        self.0
            .insert(key, Box::new(move |v| value.parse(v).map(|x| x.into())));
        self
    }
}

impl ZData for ZObject {
    type Output = Value;

    fn parse(&self, data: Value) -> Result<Self::Output, ()> {
        // TODO: Ensure this handles all validation cases -> first existing no in schema, field not present, etc.

        // TODO: Use iterator to make more efficient?
        let obj = data.as_object().ok_or(())?;
        let mut output = Map::with_capacity(obj.len());
        for (key, value) in obj {
            let value = self.0.get(&**key).ok_or(())?(value.clone())?; // TODO: Remove `value` clone!
            output.insert(key.clone(), value); // TODO: Remove `key` clone.
        }

        Ok(output.into())
    }
}

pub struct Z;

impl Z {
    pub fn string() -> ZString {
        ZString {}
    }

    pub fn number() -> ZNumber {
        ZNumber {}
    }

    pub fn object() -> ZObject {
        ZObject(BTreeMap::new())
    }

    pub fn object_struct<TBackingModel: DeserializeOwned>() {
        unimplemented!()
    }
}
