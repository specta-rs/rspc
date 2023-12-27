use std::{borrow::Cow, collections::HashMap, future::Future, pin::Pin};

use erased_serde::Deserializer;

use crate::serializer::Serializer;

pub struct RequestContext<'a> {
    // pub id: u32,
    pub arg: Option<&'a mut (dyn Deserializer<'a> + Send)>, // TODO: Remove `erased-serde` from public API
    pub result: Serializer<'a>,
}

#[derive(Default)]
pub struct Executor {
    procedures: HashMap<Cow<'static, str>, Procedure>,
}

impl Executor {
    // pub async fn execute() -> () {}

    // pub async fn execute_blocking() -> () {}

    // pub async fn execute_streaming() -> () {}

    // TODO: How can the user get a `Value` without major overhead
}

pub struct Procedure {
    // TODO: Make this private
    pub handler: Box<dyn Fn(RequestContext) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>>,
}
