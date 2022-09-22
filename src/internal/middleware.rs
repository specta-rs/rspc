use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use futures::Stream;
use serde_json::Value;

use crate::ExecError;

pub trait MiddlewareBuilder<TCtx> {
    type LayerContext: 'static;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext> + 'static;
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
        T: Middleware<Self::LayerContext> + 'static,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct Demo<TCtx, TLayerCtx>
where
    TLayerCtx: 'static,
{
    pub bruh: Box<dyn Fn(Box<dyn Middleware<TLayerCtx>>) -> Box<dyn Middleware<TCtx>>>, // TODO: Make this more generic
}

impl<TCtx, TLayerCtx> MiddlewareBuilder<TCtx> for Demo<TCtx, TLayerCtx>
where
    TLayerCtx: 'static,
{
    type LayerContext = TLayerCtx;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext> + 'static,
    {
        (self.bruh)(Box::new(next))
    }
}

pub struct BaseMiddleware<TCtx: 'static>(PhantomData<TCtx>);

impl<TCtx> BaseMiddleware<TCtx> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> MiddlewareBuilder<TCtx> for BaseMiddleware<TCtx>
where
    TCtx: Send + Sync + 'static, // TODO: `+ Send + Sync` cringe
{
    type LayerContext = TCtx;

    fn build<T>(&self, next: T) -> Box<dyn Middleware<TCtx>>
    where
        T: Middleware<Self::LayerContext> + 'static,
    {
        Box::new(ResolverLayer {
            func: move |ctx, args, kak| next.call(ctx, args, kak),
            phantom: PhantomData,
        })
    }
}

pub trait Middleware<TLayerCtx: 'static>: Send + Sync {
    fn call(&self, a: TLayerCtx, b: Value, c: KindAndKey) -> Result<LayerResult, ExecError>;
}

pub struct ResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static, // TODO: `+ Send + Sync` cringe
    T: Fn(TLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx> Middleware<TLayerCtx> for ResolverLayer<TLayerCtx, T>
where
    T: Fn(TLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync,
    TLayerCtx: Send + Sync + 'static, // TODO: `+ Send + Sync` cringe
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
    pub arg: Value,
    pub(crate) nextmw: Arc<Box<dyn Middleware<TNewLayerCtx>>>,
}

impl<TLayerCtx, TNewLayerCtx> MiddlewareContext<TLayerCtx, TNewLayerCtx>
where
    TLayerCtx: 'static,
    TNewLayerCtx: Send + 'static,
{
    pub async fn next(self, ctx: TNewLayerCtx) -> Result<Value, ExecError> {
        self.nextmw
            .call(ctx, self.arg, (self.kind, self.key))?
            .into_value()
            .await
    }
}
