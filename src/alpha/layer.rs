use std::marker::PhantomData;

use serde_json::Value;

use crate::{
    internal::{LayerResult, RequestContext},
    ExecError,
};

// TODO: Rename this so it doesn't conflict with the middleware builder struct
pub trait AlphaLayer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError>;
}
