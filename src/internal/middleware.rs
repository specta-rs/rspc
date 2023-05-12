use std::{future::Future, marker::PhantomData, pin::Pin};

use futures::Stream;
use serde_json::Value;

use crate::ExecError;

pub struct BaseMiddleware<TCtx>(PhantomData<TCtx>)
where
    TCtx: 'static;

impl<TCtx> Default for BaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TCtx> BaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

// TODO: Is this a duplicate of any type?
// TODO: Move into public API cause it might be used in middleware
#[derive(Debug, Clone)]
pub enum ProcedureKind {
    Query,
    Mutation,
    Subscription,
}

impl ProcedureKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            ProcedureKind::Query => "query",
            ProcedureKind::Mutation => "mutation",
            ProcedureKind::Subscription => "subscription",
        }
    }
}

// TODO: Maybe rename to `Request` or something else. Also move into Public API cause it might be used in middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub kind: ProcedureKind,
    pub path: String, // TODO: String slice??
}

// #[deprecated = "Going to be removed in v1.0.0. The new middleware system removes the need for this."]
pub enum ValueOrStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}

// #[deprecated = "Going to be removed in v1.0.0. The new middleware system removes the need for this."]
pub enum ValueOrStreamOrFutureStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}

// TODO: Ensure this is removed from the old one
// #[deprecated = "Going to be removed in v1.0.0. The new middleware system removes the need for this."]
pub enum LayerResult {
    Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    FutureValueOrStream(Pin<Box<dyn Future<Output = Result<ValueOrStream, ExecError>> + Send>>),
    FutureValueOrStreamOrFutureStream(
        Pin<Box<dyn Future<Output = Result<ValueOrStreamOrFutureStream, ExecError>> + Send>>,
    ),
    Ready(Result<Value, ExecError>),
}

impl LayerResult {
    pub async fn into_value_or_stream(self) -> Result<ValueOrStream, ExecError> {
        match self {
            LayerResult::Stream(stream) => Ok(ValueOrStream::Stream(stream)),
            LayerResult::Future(fut) => Ok(ValueOrStream::Value(fut.await?)),
            LayerResult::FutureValueOrStream(fut) => Ok(fut.await?),
            LayerResult::FutureValueOrStreamOrFutureStream(fut) => Ok(match fut.await? {
                ValueOrStreamOrFutureStream::Value(val) => ValueOrStream::Value(val),
                ValueOrStreamOrFutureStream::Stream(stream) => ValueOrStream::Stream(stream),
            }),
            LayerResult::Ready(res) => Ok(ValueOrStream::Value(res?)),
        }
    }
}
