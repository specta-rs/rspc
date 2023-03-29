use std::{
    any::Any,
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project::pin_project;
use serde_json::Value;

use crate::{
    ExecError, MiddlewareContext, MiddlewareFutOrSomething, MiddlewareLike, NoShot, PinnedOption,
};

pub trait MiddlewareBuilderLike<TCtx: 'static> {
    type LayerContext: 'static;
    type LayerResult<T>: Layer<TCtx>
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
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

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct MiddlewareLayerWithNext();

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

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext> + Sync,
    {
        let func = self.mw.explode();

        self.middleware.build(MiddlewareLayer {
            next: Arc::new(next), // Avoiding `Arc`
            mw: func, // self.mw.clone(),  // TODO: Avoid `Clone` bound when `build` takes `self`
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
    // mw: TNewMiddleware,
    mw: TNewMiddleware::Fn,
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
    type Fut = MiddlewareFutOrSomething<
        TNewMiddleware::State,
        TLayerCtx,
        TNewLayerCtx,
        TNewMiddleware::Fut2,
        TMiddleware,
    >; // TNewMiddleware::Fut<TMiddleware>;
    type Call2Fut = NoShot<TNewLayerCtx, TMiddleware>;

    fn call(&self, ctx: TLayerCtx, input: Value, req: RequestContext) -> Self::Fut {
        // TODO: Don't take ownership of `self.next` to avoid needing it to be `Arc`ed

        // self.mw.handle(ctx, input, req, &self.next)

        let handler = (self.mw)(MiddlewareContext {
            state: (),
            ctx,
            input,
            req,
            phantom: PhantomData,
        });

        // TODO: Avoid taking ownership of `next`
        MiddlewareFutOrSomething(PinnedOption::Some(handler), self.next.clone())
    }

    fn call2(
        &self,
        ctx: Box<dyn Any + Send + 'static>,
        value: Value,
        req: RequestContext,
    ) -> Self::Call2Fut {
        let fut = self.next.call(*ctx.downcast().unwrap(), value, req);
        NoShot(PinnedOption::Some(fut))
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

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>,
    {
        next
    }
}

// TODO: Rename this so it doesn't conflict with the middleware builder struct
// TODO: Document the types and functions so they make sense
pub trait Layer<TLayerCtx: 'static>: DynLayer<TLayerCtx> + Send + Sync + 'static {
    type Fut: Future<Output = Result<ValueOrStreamOrFut2, ExecError>> + Send + 'static; // TODO: This may need lifetime back but let's remove it for now
                                                                                        // type Fut2: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static;
    type Call2Fut: Future<Output = Result<ValueOrStreamOrFut2, ExecError>> + Send + 'static;

    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Self::Fut;

    fn call2(
        &self,
        ctx: Box<dyn Any + Send + 'static>,
        value: Value,
        req: RequestContext,
    ) -> Self::Call2Fut {
        unreachable!(); // TODO: Don't do this
    }

    fn erase(self) -> Box<dyn DynLayer<TLayerCtx>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// TODO: Does this need lifetime?
pub type FutureValueOrStream<'a> =
    Pin<Box<dyn Future<Output = Result<ValueOrStream, ExecError>> + Send + 'a>>;

pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn dyn_call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream<'a>, ExecError>;
}

impl<TLayerCtx: Send + 'static, L: Layer<TLayerCtx>> DynLayer<TLayerCtx> for L {
    fn dyn_call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream<'a>, ExecError> {
        Ok(Box::pin(async move {
            match Layer::call(self, a, b, c).await? {
                ValueOrStreamOrFut2::Value(x) => Ok(ValueOrStream::Value(x)),
                ValueOrStreamOrFut2::TheSolution(ctx, input, req) => {
                    let mut fut = self.call2(ctx, input, req).await?;

                    // TODO: This will keep calling the first middleware (`self`) whenever any middleware wants to call it's own next one
                    // TODO: The problem with `Self::Fut2` being created on each layer is that by doing so we need `&Middleware` on that specific layer which means cringe `Arc`
                    // TODO: Finally remove `Arc` from middleware and `Box<dyn Any>` from `ctx`
                    loop {
                        match fut {
                            ValueOrStreamOrFut2::Value(x) => break Ok(ValueOrStream::Value(x)),
                            ValueOrStreamOrFut2::TheSolution(ctx, input, req) => {
                                fut = self.call2(ctx, input, req).await?;

                                // todo!();
                            }
                            ValueOrStreamOrFut2::Stream(x) => break Ok(ValueOrStream::Stream(x)),
                        }
                    }
                }
                ValueOrStreamOrFut2::Stream(x) => Ok(ValueOrStream::Stream(x)),
            }
        }))
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
    type Call2Fut = Ready<Result<ValueOrStreamOrFut2, ExecError>>; // Unused

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
    type Output = Result<ValueOrStreamOrFut2, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            // TODO: Simplify this a bit
            Poll::Ready(Ok(ValueOrStream::Value(v))) => {
                Poll::Ready(Ok(ValueOrStreamOrFut2::Value(v)))
            }
            Poll::Ready(Ok(ValueOrStream::Stream(s))) => {
                Poll::Ready(Ok(ValueOrStreamOrFut2::Stream(s)))
            }
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

// TODO: Replace this with `ValueOrStreamOrFut2`
pub enum ValueOrStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    // TODO: Rename this
    // TheSolution(Box<dyn Any + Send + 'static>, Value, RequestContext),
}

pub enum ValueOrStreamOrFut2 {
    Value(Value),
    // TODO: Rename this
    TheSolution(Box<dyn Any + Send + 'static>, Value, RequestContext),
    // TODO: Take this type in as a generic
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}
