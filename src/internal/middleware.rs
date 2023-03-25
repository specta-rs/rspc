use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project::pin_project;
use serde_json::Value;

use crate::{ExecError, MiddlewareLike};

pub trait MiddlewareBuilderLike<TCtx: 'static> {
    type LayerContext: 'static;
    type LayerResult<T>: Layer<TCtx>
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(&self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>;
}

pub struct MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TCtx: 'static,
    TLayerCtx: 'static,
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
    type LayerResult<T> = TMiddleware::LayerResult<TIncomingMiddleware::LayerResult<T>>
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(&self, next: T) -> Self::LayerResult<T>
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
    type LayerResult<T> = TMiddleware::LayerResult<MiddlewareLayer<TLayerCtx, TNewLayerCtx, T, TNewMiddleware>> // TODO
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(&self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext> + Sync,
    {
        self.middleware.build(MiddlewareLayer {
            next: Arc::new(next), // Avoiding `Arc`
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
    type Fut = TNewMiddleware::Fut<TMiddleware>;

    fn call(&self, ctx: TLayerCtx, input: Value, req: RequestContext) -> Self::Fut {
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

impl<TCtx> MiddlewareBuilderLike<TCtx> for BaseMiddleware<TCtx>
where
    TCtx: Send + 'static,
{
    type LayerContext = TCtx;
    type LayerResult<T> = T
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(&self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>,
    {
        next
    }
}

// TODO: Rename this so it doesn't conflict with the middleware builder struct
pub trait Layer<TLayerCtx: 'static>: DynLayer<TLayerCtx> + Send + Sync + 'static {
    type Fut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static;

    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Self::Fut;

    fn erase(self) -> Box<dyn DynLayer<TLayerCtx>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

pub type FutureValueOrStream =
    Pin<Box<dyn Future<Output = Result<ValueOrStream, ExecError>> + Send>>;

pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn dyn_call(
        &self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream, ExecError>;
}

impl<TLayerCtx: 'static, L: Layer<TLayerCtx>> DynLayer<TLayerCtx> for L {
    fn dyn_call(
        &self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream, ExecError> {
        Ok(Box::pin(Layer::call(self, a, b, c)))
    }
}

pub struct ResolverLayer<TLayerCtx, T, TFut>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx, TFut> Layer<TLayerCtx> for ResolverLayer<TLayerCtx, T, TFut>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static,
{
    type Fut = ResolverLayerFut<TFut>;

    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Self::Fut {
        ResolverLayerFut((self.func)(a, b, c))
    }
}

#[pin_project(project = ResolverLayerFutProj)]
pub struct ResolverLayerFut<
    TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static,
>(#[pin] TFut);

impl<TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static> Future
    for ResolverLayerFut<TFut>
{
    type Output = Result<ValueOrStream, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            // TODO: Simplify this a bit
            Poll::Ready(Ok(ValueOrStream::Value(v))) => Poll::Ready(Ok(ValueOrStream::Value(v))),
            Poll::Ready(Ok(ValueOrStream::Stream(s))) => Poll::Ready(Ok(ValueOrStream::Stream(s))),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
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

// TODO: Avoid using `Ready<T>` for top layer and instead store as `Value` so the procedure can be quick as fuck???

// TODO: Move this into the file with `dyn_call` and stop using it in this file
pub enum ValueOrStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}
