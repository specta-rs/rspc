use std::{borrow::Cow, collections::HashMap, future::Future, pin::Pin};

use crate::{serializer::Serializer, Format, Task};

pub struct RequestContext<'a> {
    // pub id: u32,
    // pub arg: Option<&'a mut (dyn Deserializer<'a> + Send)>, // TODO: Remove `erased-serde` from public API
    pub result: Serializer<'a>,
}

pub type Procedure = Box<dyn Fn(RequestContext) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>>;

#[derive(Default)]
pub struct Executor {
    procedures: HashMap<Cow<'static, str>, Procedure>,
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.procedures.contains_key(name)
    }

    pub fn insert(&mut self, name: Cow<'static, str>, procedure: Procedure) {
        self.procedures.insert(name.into(), procedure);
    }

    pub fn remove(&mut self, name: &str) -> Option<Procedure> {
        self.procedures.remove(name)
    }

    pub fn len(&self) -> usize {
        self.procedures.len()
    }

    pub async fn execute<F: Format>(name: &str) -> Task<F> {
        todo!();
    }
}
