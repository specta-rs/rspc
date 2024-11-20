use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use futures::Stream;
use serde_json::Value;

use crate::{ExecError, MiddlewareLike};

pub trait MiddlewareBuilderLike<TCtx> {
    type LayerContext: 'static;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext>;
}

pub struct MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub middleware2: TIncomingMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware> MiddlewareBuilderLike<TCtx>
    for MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TCtx: 'static,
    TLayerCtx: 'static,
    TNewLayerCtx: 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    type LayerContext = TNewLayerCtx;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext>,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub mw: TNewMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx, TNewLayerCtx)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware> MiddlewareBuilderLike<TCtx>
    for MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    type LayerContext = TNewLayerCtx;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext> + Sync,
    {
        self.middleware.build(MiddlewareLayer {
            next: Arc::new(next),
            mw: self.mw.clone(),
            phantom: PhantomData,
        })
    }
}

pub struct MiddlewareLayer<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TMiddleware: Layer<TNewLayerCtx> + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    next: Arc<TMiddleware>, // TODO: Avoid arcing this if possible
    mw: TNewMiddleware,
    phantom: PhantomData<(TLayerCtx, TNewLayerCtx)>,
}

impl<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware> Layer<TLayerCtx>
    for MiddlewareLayer<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: Layer<TNewLayerCtx> + Sync + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<LayerResult, ExecError> {
        self.mw.handle(ctx, input, req, self.next.clone())
    }
}

pub struct BaseMiddleware<TCtx>(PhantomData<TCtx>)
where
    TCtx: 'static;

impl<TCtx> Default for BaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> MiddlewareBuilderLike<TCtx> for BaseMiddleware<TCtx>
where
    TCtx: Send + 'static,
{
    type LayerContext = TCtx;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext>,
    {
        Box::new(next)
    }
}

// TODO: Rename this so it doesn't conflict with the middleware builder struct
pub trait Layer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError>;
}

pub struct ResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<LayerResult, ExecError>
        + Send
        + Sync
        + 'static,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx> Layer<TLayerCtx> for ResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<LayerResult, ExecError>
        + Send
        + Sync
        + 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError> {
        (self.func)(a, b, c)
    }
}

impl<TLayerCtx> Layer<TLayerCtx> for Box<dyn Layer<TLayerCtx> + 'static>
where
    TLayerCtx: 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError> {
        (**self).call(a, b, c)
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

pub enum ValueOrStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}

pub enum ValueOrStreamOrFutureStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}

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
