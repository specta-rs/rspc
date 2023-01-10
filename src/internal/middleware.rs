use std::{marker::PhantomData, sync::Arc};

use futures::{future::BoxFuture, stream::BoxStream};
use serde::Serialize;
use serde_json::Value;
use specta::Type;

use crate::{internal::MiddlewareLike, ExecError};

pub trait MiddlewareBuilderLike<TCtx> {
    type LayerContext: Send + Sync + 'static;

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
    TNewLayerCtx: Send + Sync + 'static,
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
    fn call(&self, ctx: TLayerCtx, input: Value, req: RequestContext) -> ExecResult<LayerFuture> {
        Ok(self.mw.handle(ctx, input, req, self.next.clone()))
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
    TCtx: Send + Sync + 'static,
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
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerFuture, ExecError>;
}

pub struct ResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<LayerFuture, ExecError>
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
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<LayerFuture, ExecError>
        + Send
        + Sync
        + 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerFuture, ExecError> {
        (self.func)(a, b, c)
    }
}

impl<TLayerCtx> Layer<TLayerCtx> for Box<dyn Layer<TLayerCtx> + 'static>
where
    TLayerCtx: 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerFuture, ExecError> {
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

pub type ExecResult<T> = Result<T, ExecError>;

pub enum RequestFuture {
    Ready(ExecResult<Value>),
    Future(BoxFuture<'static, ExecResult<Value>>),
}

impl RequestFuture {
    pub async fn exec(self) -> ExecResult<Value> {
        match self {
            Self::Ready(res) => res,
            Self::Future(fut) => fut.await,
        }
    }
}

pub trait RequestResultData: Serialize + Type + Send + 'static {}
impl<T: Serialize + Type + Send + 'static> RequestResultData for T {}

pub enum TypedRequestFuture<T> {
    Ready(ExecResult<T>),
    Future(BoxFuture<'static, ExecResult<T>>),
}

impl<T> TypedRequestFuture<T>
where
    T: RequestResultData,
{
    pub fn to_request_future(self) -> RequestFuture {
        match self {
            Self::Ready(val) => RequestFuture::Ready({
                val.map(|v| serde_json::to_value(v).map_err(ExecError::DeserializingArgErr))
                    .and_then(|v| v)
            }),
            Self::Future(fut) => RequestFuture::Future(Box::pin(async move {
                fut.await
                    .map(serde_json::to_value)?
                    .map_err(ExecError::DeserializingArgErr)
            })),
        }
    }

    pub async fn exec(self) -> ExecResult<T> {
        match self {
            Self::Ready(res) => res,
            Self::Future(fut) => fut.await,
        }
    }
}

pub type StreamItem = ExecResult<Value>;
pub type StreamFuture = BoxStream<'static, StreamItem>;

pub enum LayerFuture {
    Request(RequestFuture),
    Stream(StreamFuture),
    Wrapped(BoxFuture<'static, ExecResult<LayerFuture>>),
}

pub enum LayerReturn {
    Request(Value),
    Stream(BoxStream<'static, StreamItem>),
}

impl LayerFuture {
    pub fn into_layer_return(self) -> BoxFuture<'static, ExecResult<LayerReturn>> {
        Box::pin(async {
            match self {
                Self::Request(req) => req.exec().await.map(LayerReturn::Request),
                Self::Stream(stream) => Ok(LayerReturn::Stream(stream)),
                Self::Wrapped(fut) => fut.await?.into_layer_return().await,
            }
        })
    }
}

impl From<RequestFuture> for LayerFuture {
    fn from(v: RequestFuture) -> Self {
        Self::Request(v)
    }
}

impl From<StreamFuture> for LayerFuture {
    fn from(v: StreamFuture) -> Self {
        Self::Stream(v)
    }
}
