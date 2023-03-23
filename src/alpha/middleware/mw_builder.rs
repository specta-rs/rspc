use std::{future::Future, marker::PhantomData};

use serde_json::Value;

use crate::internal::RequestContext;

// TODO: Remove this
pub trait MiddlewareResult {}
