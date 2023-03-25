use pin_project::pin_project;
use serde_json::Value;
use std::{
    any::Any,
    future::{Future, Ready},
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{
    internal::{Layer, RequestContext, ValueOrStream, ValueOrStreamOrFut2},
    ExecError,
};

pub trait MiddlewareLike<TLayerCtx>: Clone {
    type State: Clone + Send + Sync + 'static;
    type NewCtx: Send + 'static;

    type Fut<TMiddleware: Layer<Self::NewCtx>>: Future<Output = Result<ValueOrStreamOrFut2<Self::Fut2<TMiddleware>>, ExecError>>
        + Send
        + 'static;
    type Fut2<TMiddleware: Layer<Self::NewCtx>>: Future<Output = Result<ValueOrStream, ExecError>>
        + Send
        + 'static;

    // TODO: Rename `exec`
    fn handle<TMiddleware: Layer<Self::NewCtx>>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        // TODO: Avoid `Arc` here
        next: Arc<TMiddleware>,
    ) -> Self::Fut<TMiddleware>;
}
pub struct MiddlewareContext<TLayerCtx, TNewCtx = TLayerCtx, TState = ()>
where
    TState: Send,
{
    pub state: TState,
    pub input: Value,
    pub ctx: TNewCtx,
    pub req: RequestContext,
    pub phantom: PhantomData<TLayerCtx>,
}

// This will match were TState is the default (`()`) so it shouldn't let you call it if you've already swapped the generic
impl<TLayerCtx, TNewCtx> MiddlewareContext<TLayerCtx, TNewCtx, ()>
where
    TLayerCtx: Send,
{
    pub fn with_state<TState>(self, state: TState) -> MiddlewareContext<TLayerCtx, TNewCtx, TState>
    where
        TState: Send,
    {
        MiddlewareContext {
            state,
            input: self.input,
            ctx: self.ctx,
            req: self.req,
            phantom: PhantomData,
        }
    }
}

// This will match were TNewCtx is the default (`TCtx`) so it shouldn't let you call it if you've already swapped the generic
impl<TLayerCtx, TState> MiddlewareContext<TLayerCtx, TLayerCtx, TState>
where
    TLayerCtx: Send,
    TState: Send,
{
    pub fn with_ctx<TNewCtx>(
        self,
        new_ctx: TNewCtx,
    ) -> MiddlewareContext<TLayerCtx, TNewCtx, TState> {
        MiddlewareContext {
            state: self.state,
            input: self.input,
            ctx: new_ctx,
            req: self.req,
            phantom: PhantomData,
        }
    }

    #[cfg(feature = "alpha")] // TODO: Stablise
    pub fn map_ctx<TNewCtx>(
        self,
        new_ctx: impl FnOnce(TLayerCtx) -> TNewCtx,
    ) -> MiddlewareContext<TLayerCtx, TNewCtx, TState> {
        MiddlewareContext {
            state: self.state,
            input: self.input,
            ctx: new_ctx(self.ctx),
            req: self.req,
            phantom: PhantomData,
        }
    }
}

#[cfg(feature = "alpha")] // TODO: Stablise
impl<TLayerCtx, TNewCtx, TState> MiddlewareContext<TLayerCtx, TNewCtx, TState>
where
    TLayerCtx: Send,
    TNewCtx: Send,
    TState: Send,
{
    pub fn map_arg(
        self,
        // arg: impl FnOnce(TLayerCtx) -> TNewCtx,
    ) -> MiddlewareContext<TLayerCtx, TNewCtx, TState> {
        MiddlewareContext {
            state: self.state,
            input: self.input,
            ctx: self.ctx,
            req: self.req,
            phantom: PhantomData,
        }
    }
}

pub struct Middleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    handler: THandlerFunc,
    phantom: PhantomData<(TState, TLayerCtx)>,
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut> Clone
    for Middleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            phantom: PhantomData,
        }
    }
}

pub struct MiddlewareBuilder<TLayerCtx>(pub PhantomData<TLayerCtx>)
where
    TLayerCtx: Send;

impl<TLayerCtx> MiddlewareBuilder<TLayerCtx>
where
    TLayerCtx: Send,
{
    pub fn middleware<TState, TNewCtx, THandlerFunc, THandlerFut>(
        &self,
        handler: THandlerFunc,
    ) -> Middleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
    where
        TState: Send,
        THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
        THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
            + Send
            + 'static,
    {
        Middleware {
            handler,
            phantom: PhantomData,
        }
    }

    // // #[cfg(feature = "alpha")] // TODO: Stablise
    // pub fn args<TMiddlewareMapper: MiddlewareArgMapper>(
    //     &self,
    // ) -> MiddlewareBuilderWithArgs<TLayerCtx, TMiddlewareMapper> {
    //     MiddlewareBuilderWithArgs(PhantomData)
    // }
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
    Middleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    pub fn resp<TRespHandlerFunc, TRespHandlerFut>(
        self,
        handler: TRespHandlerFunc,
    ) -> MiddlewareWithResponseHandler<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TRespHandlerFunc,
        TRespHandlerFut,
    >
    where
        TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
        TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
    {
        MiddlewareWithResponseHandler {
            handler: self.handler,
            resp_handler: handler,
            phantom: PhantomData,
        }
    }
}

pub struct MiddlewareWithResponseHandler<
    TState,
    TLayerCtx,
    TNewCtx,
    THandlerFunc,
    THandlerFut,
    TRespHandlerFunc,
    TRespHandlerFut,
> where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
{
    handler: THandlerFunc,
    resp_handler: TRespHandlerFunc,
    phantom: PhantomData<(TState, TLayerCtx)>,
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TRespHandlerFunc, TRespHandlerFut> Clone
    for MiddlewareWithResponseHandler<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TRespHandlerFunc,
        TRespHandlerFut,
    >
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            resp_handler: self.resp_handler.clone(),
            phantom: PhantomData,
        }
    }
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut> MiddlewareLike<TLayerCtx>
    for Middleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send + 'static,
    TNewCtx: Send + 'static,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    type State = TState;
    type NewCtx = TNewCtx;
    // TODO: Change this back
    type Fut<TMiddleware: Layer<Self::NewCtx>> =
        MiddlewareFutOrSomething<TState, TLayerCtx, TNewCtx, THandlerFut, TMiddleware>;
    type Fut2<TMiddleware: Layer<Self::NewCtx>> = NoShot<TNewCtx, TMiddleware>;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Self::Fut<TMiddleware> {
        let handler = (self.handler)(MiddlewareContext {
            state: (),
            ctx,
            input,
            req,
            phantom: PhantomData,
        });

        // TODO: Avoid taking ownership of `next`
        MiddlewareFutOrSomething(PinnedOption::Some(handler), next)
    }
}

#[pin_project(project = PinnedOptionProj)]
pub(crate) enum PinnedOption<T> {
    Some(#[pin] T),
    None,
}

// TODO: Cleanup generics on this
#[pin_project(project = MiddlewareFutOrSomethingProj)]
pub struct MiddlewareFutOrSomething<
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send + 'static,
    TNewCtx: Send + 'static,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TMiddleware: Layer<TNewCtx> + 'static,
>(#[pin] PinnedOption<THandlerFut>, Arc<TMiddleware>); // TODO: Remove `.1`

impl<
        TState: Clone + Send + Sync + 'static,
        TLayerCtx: Send + 'static,
        TNewCtx: Send + 'static,
        THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
            + Send
            + 'static,
        TMiddleware: Layer<TNewCtx> + 'static,
    > Future for MiddlewareFutOrSomething<TState, TLayerCtx, TNewCtx, THandlerFut, TMiddleware>
{
    type Output = Result<ValueOrStreamOrFut2<NoShot<TNewCtx, TMiddleware>>, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match this.0.as_mut().project() {
            PinnedOptionProj::Some(fut) => match fut.poll(cx) {
                Poll::Ready(Ok(handler)) => {
                    this.0.set(PinnedOption::None);
                    // let fut = this.1.call(handler.ctx, handler.input, handler.req); // TODO

                    let ctx: Box<dyn Any + Send + 'static> = Box::new(handler.ctx); // TODO: Add generics so this isn't allocated

                    return Poll::Ready(Ok(ValueOrStreamOrFut2::TheSolution(
                        ctx,
                        handler.input,
                        handler.req,
                    )));
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(ExecError::ErrResolverError(e))),
                Poll::Pending => return Poll::Pending,
            },
            PinnedOptionProj::None => {}
        }

        unreachable!()
    }
}

// TODO: Move into another file where it's used
// TODO: Should I remove this?
#[pin_project(project = NoShotProj)]
pub struct NoShot<TNewCtx: Send + 'static, TMiddleware: Layer<TNewCtx> + 'static>(
    #[pin] pub(crate) PinnedOption<TMiddleware::Fut>,
    #[pin] pub(crate) PinnedOption<TMiddleware::Fut2>,
);

impl<TNewCtx: Send + 'static, TMiddleware: Layer<TNewCtx> + 'static> Future
    for NoShot<TNewCtx, TMiddleware>
{
    type Output = Result<ValueOrStream, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match this.0.as_mut().project() {
            PinnedOptionProj::Some(fut) => match fut.poll(cx) {
                Poll::Ready(Ok(result)) => {
                    this.0.set(PinnedOption::None);

                    match result {
                        ValueOrStreamOrFut2::Value(v) => {
                            return Poll::Ready(Ok(ValueOrStream::Value(v)))
                        }
                        ValueOrStreamOrFut2::Stream(s) => {
                            return Poll::Ready(Ok(ValueOrStream::Stream(s)))
                        }
                        ValueOrStreamOrFut2::Fut2(fut) => {
                            this.1.set(PinnedOption::Some(fut));
                        }
                        ValueOrStreamOrFut2::TheSolution(ctx, input, req) => {
                            // return Poll::Ready(ValueOrStream)
                            todo!();
                        }
                    }
                }
                Poll::Ready(Err(e)) => {
                    this.0.set(PinnedOption::None);
                    return Poll::Ready(Err(e));
                }
                Poll::Pending => return Poll::Pending,
            },
            PinnedOptionProj::None => {}
        }

        match this.1.as_mut().project() {
            PinnedOptionProj::Some(fut) => match fut.poll(cx) {
                Poll::Ready(result) => {
                    this.1.set(PinnedOption::None);
                    return Poll::Ready(result);
                }
                Poll::Pending => return Poll::Pending,
            },
            PinnedOptionProj::None => {}
        }

        unreachable!()
    }
}

// TODO: Removing this?
pub(crate) enum FutOrValue<T: Future<Output = Result<Value, crate::Error>>> {
    Fut(T),
    Value(Result<Value, ExecError>),
}

// // TODO: Deduplicate this with `MiddlewareLike<TLayerCtx> for Middleware` that is basically the same minus the oncomplete bit. Same with deduplicating the futures
// impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TRespHandlerFunc, TRespHandlerFut>
//     MiddlewareLike<TLayerCtx>
//     for MiddlewareWithResponseHandler<
//         TState,
//         TLayerCtx,
//         TNewCtx,
//         THandlerFunc,
//         THandlerFut,
//         TRespHandlerFunc,
//         TRespHandlerFut,
//     >
// where
//     TState: Clone + Send + Sync + 'static,
//     TLayerCtx: Send + 'static,
//     TNewCtx: Send + 'static,
//     THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
//     THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
//         + Send
//         + 'static,
//     TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
//     TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
// {
//     type State = TState;
//     type NewCtx = TNewCtx;

//     type Fut<TMiddleware: Layer<Self::NewCtx>> =
//         MiddlewareFutOrSomething<TState, TLayerCtx, TNewCtx, THandlerFut, TMiddleware>;

//     fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
//         &self,
//         ctx: TLayerCtx,
//         input: Value,
//         req: RequestContext,
//         next: Arc<TMiddleware>,
//     ) -> Self::Fut<TMiddleware> {
//         let handler = (self.handler)(MiddlewareContext {
//             state: (),
//             ctx,
//             input,
//             req,
//             // new_ctx: None,
//             phantom: PhantomData,
//         });

//         let f = self.resp_handler.clone(); // TODO: Runtime clone is bad. Avoid this!

//         // Ok(LayerResult::FutureValueOrStreamOrFutureStream(Box::pin(
//         //     async move {
//         //         let handler = handler.await?;

//         //         Ok(
//         //             match next.call(handler.ctx, handler.input, handler.req).await? {
//         //                 // ValueOrStream::Value(v) => ValueOrStream::Value(f(handler.state, v).await?),
//         //                 // ValueOrStream::Stream(s) => {
//         //                 //     ValueOrStream::Stream(Box::pin(s.then(move |v| {
//         //                 //         let v = match v {
//         //                 //             Ok(v) => FutOrValue::Fut(f(handler.state.clone(), v)),
//         //                 //             e => FutOrValue::Value(e),
//         //                 //         };

//         //                 //         async move {
//         //                 //             match v {
//         //                 //                 FutOrValue::Fut(fut) => {
//         //                 //                     fut.await.map_err(ExecError::ErrResolverError)
//         //                 //                 }
//         //                 //                 FutOrValue::Value(v) => v,
//         //                 //             }
//         //                 //         }
//         //                 //     })))
//         //                 // }
//         //                 _ => todo!(),
//         //             },
//         //         )
//         //     },
//         // )))

//         todo!("Middleware we `.resp` currently isn't wired up to work but it should be easy to fix in future!");
//     }
// }
