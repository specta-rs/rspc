use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use futures::Stream;
use serde_json::Value;

use crate::ExecError;

pub trait MiddlewareBuilder<TCtx> {
    type LayerContext: 'static;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext>;
}

pub struct MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TMiddleware: MiddlewareBuilder<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: MiddlewareBuilder<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub middleware2: TIncomingMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware> MiddlewareBuilder<TCtx>
    for MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TCtx: 'static,
    TLayerCtx: 'static,
    TNewLayerCtx: 'static,
    TMiddleware: MiddlewareBuilder<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: MiddlewareBuilder<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    type LayerContext = TNewLayerCtx;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext>,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct MiddlewareLayer<TCtx, TLayerCtx, TNewLayerCtx, TFut, TMiddleware>
where
    TCtx: Send + 'static,
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TFut: Future<Output = Result<Value, ExecError>> + Send + 'static,
    TMiddleware: MiddlewareBuilder<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    pub middleware: TMiddleware,
    pub handler: fn(MiddlewareContext<TLayerCtx, TNewLayerCtx>) -> TFut,
    pub phantom: PhantomData<(TCtx, TLayerCtx, TNewLayerCtx, TFut)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TFut, TMiddleware> MiddlewareBuilder<TCtx>
    for MiddlewareLayer<TCtx, TLayerCtx, TNewLayerCtx, TFut, TMiddleware>
where
    TCtx: Send + 'static,
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TFut: Future<Output = Result<Value, ExecError>> + Send + 'static,
    TMiddleware: MiddlewareBuilder<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    type LayerContext = TNewLayerCtx;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext> + Sync,
    {
        self.middleware.build(Bruh {
            next: Arc::new(next),
            handler: self.handler,
        })
    }
}

pub struct Bruh<TLayerCtx, TNewLayerCtx, TFut, TMiddleware>
where
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TFut: Future<Output = Result<Value, ExecError>> + Send + 'static,
    TMiddleware: Middleware<TNewLayerCtx> + 'static,
{
    next: Arc<TMiddleware>, // TODO: Avoid arcing this if possible
    handler: fn(MiddlewareContext<TLayerCtx, TNewLayerCtx>) -> TFut,
}

impl<TLayerCtx, TNewLayerCtx, TFut, TMiddleware> Middleware<TLayerCtx>
    for Bruh<TLayerCtx, TNewLayerCtx, TFut, TMiddleware>
where
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TFut: Future<Output = Result<Value, ExecError>> + Send + 'static,
    TMiddleware: Middleware<TNewLayerCtx> + Sync + 'static,
{
    fn call(&self, ctx: TLayerCtx, input: Value, c: KindAndKey) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::FutureStreamOrValue(Box::pin((self.handler)(
            MiddlewareContext::<TLayerCtx, TNewLayerCtx> {
                key: c.1,
                kind: c.0,
                ctx,
                input,
                nextmw: self.next.clone(),
            },
        ))))
    }
}

pub struct BaseMiddleware<TCtx>(PhantomData<TCtx>)
where
    TCtx: 'static;

impl<TCtx> BaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> MiddlewareBuilder<TCtx> for BaseMiddleware<TCtx>
where
    TCtx: Send + 'static,
{
    type LayerContext = TCtx;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext>,
    {
        Box::new(next)
    }
}

pub trait Middleware<TLayerCtx: 'static>: Send + Sync + 'static {
    fn call(&self, a: TLayerCtx, b: Value, c: KindAndKey) -> Result<LayerResult, ExecError>;
}

pub struct ResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync + 'static,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx> Middleware<TLayerCtx> for ResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync + 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: KindAndKey) -> Result<LayerResult, ExecError> {
        (self.func)(a, b, c)
    }
}

impl<TLayerCtx> Middleware<TLayerCtx> for Box<dyn Middleware<TLayerCtx> + 'static>
where
    TLayerCtx: 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: KindAndKey) -> Result<LayerResult, ExecError> {
        (**self).call(a, b, c)
    }
}

// BREAK

// #[deprecated]
pub struct OperationKey();

// #[deprecated]
pub struct OperationKind();

// #[deprecated]
pub type KindAndKey = (OperationKind, OperationKey);

pub enum LayerResult {
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    FutureStreamOrValue(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    Ready(Result<Value, ExecError>),
}

impl LayerResult {
    // TODO: Probs just use `Into<Value>` trait instead
    pub(crate) async fn into_value(self) -> Result<Value, ExecError> {
        match self {
            LayerResult::Stream(_stream) => todo!(), // Ok(StreamOrValue::Stream(stream)),
            LayerResult::Future(fut) => Ok(fut.await?),
            LayerResult::FutureStreamOrValue(fut) => Ok(fut.await?),
            LayerResult::Ready(res) => Ok(res?),
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
    pub input: Value,
    pub(crate) nextmw: Arc<dyn Middleware<TNewLayerCtx>>,
}

impl<TLayerCtx, TNewLayerCtx> MiddlewareContext<TLayerCtx, TNewLayerCtx>
where
    TLayerCtx: 'static,
    TNewLayerCtx: Send + 'static,
{
    pub async fn next(self, ctx: TNewLayerCtx) -> Result<Value, ExecError> {
        self.nextmw
            .call(ctx, self.input, (self.kind, self.key))?
            .into_value()
            .await
    }
}
