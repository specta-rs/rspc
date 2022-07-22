use serde_json::Value;

use crate::{Error, ResolverResult};

pub struct Procedure<TCtx> {
    pub name: String,
    pub exec: Box<dyn Fn(TCtx, Value) -> Result<ResolverResult, Error>>,
}
