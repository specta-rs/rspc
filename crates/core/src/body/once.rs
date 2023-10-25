use std::{future::Future, pin::Pin};

use futures::stream::Once;
use serde_json::Value;

use crate::error::ExecError;

use super::Body;

impl Body for Pin<Box<dyn Body + Send + '_>> {}

impl<Fut: Future<Output = Result<Value, ExecError>>> Body for Once<Fut> {}
