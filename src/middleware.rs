use std::{fmt, future::Future, pin::Pin, sync::Arc};

use futures::Stream;
use serde_json::Value;

use crate::{ExecError, KindAndKey, OperationKey, OperationKind};

pub type NextMiddleware<TLayerCtx> =
    Box<dyn Fn(TLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync>;

pub type FirstMiddleware<TCtx> =
    Box<dyn Fn(TCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync>;

pub enum LayerResult {
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    FutureStreamOrValue(Pin<Box<dyn Future<Output = Result<StreamOrValue, ExecError>> + Send>>),
    Ready(Result<Value, ExecError>),
}

impl LayerResult {
    pub(crate) async fn into_stream_or_value(self) -> Result<StreamOrValue, ExecError> {
        match self {
            LayerResult::Stream(stream) => Ok(StreamOrValue::Stream(stream)),
            LayerResult::Future(fut) => Ok(StreamOrValue::Value(fut.await?)),
            LayerResult::FutureStreamOrValue(fut) => Ok(fut.await?),
            LayerResult::Ready(res) => Ok(StreamOrValue::Value(res?)),
        }
    }
}

pub enum StreamOrValue {
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    Value(Value),
}

impl fmt::Debug for StreamOrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamOrValue::Stream(_) => write!(f, "StreamOrValue::Stream(_)"),
            StreamOrValue::Value(value) => write!(f, "StreamOrValue::Value({:?})", value),
        }
    }
}

pub struct MiddlewareContext<TLayerCtx, TNewLayerCtx>
where
    TNewLayerCtx: Send,
{
    pub key: OperationKey,
    pub kind: OperationKind,
    pub ctx: TLayerCtx,
    pub arg: Value,
    pub(crate) nextmw: Arc<
        dyn Fn(TNewLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync,
    >,
}

impl<TLayerCtx, TNewLayerCtx> MiddlewareContext<TLayerCtx, TNewLayerCtx>
where
    TNewLayerCtx: Send,
{
    pub async fn next(self, ctx: TNewLayerCtx) -> Result<StreamOrValue, ExecError> {
        (self.nextmw)(ctx, self.arg, (self.kind, self.key))?
            .into_stream_or_value()
            .await
    }
}
