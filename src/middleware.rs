use futures::StreamExt;
use serde_json::Value;
use std::{future::Future, marker::PhantomData, sync::Arc};

use crate::{
    internal::{Layer, LayerResult, RequestContext, ValueOrStream, ValueOrStreamOrFutureStream},
    ExecError,
};

pub trait MiddlewareLike<TLayerCtx>: Clone {
    type State: Clone + Send + Sync + 'static;
    type NewCtx: Send + 'static;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError>;
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
    TLayerCtx: Send,
    TNewCtx: Send + 'static,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    type State = TState;
    type NewCtx = TNewCtx;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError> {
        let handler = (self.handler)(MiddlewareContext {
            state: (),
            ctx,
            input,
            req,
            phantom: PhantomData,
        });

        Ok(LayerResult::FutureValueOrStream(Box::pin(async move {
            let handler = handler.await?;
            next.call(handler.ctx, handler.input, handler.req)?
                .into_value_or_stream()
                .await
        })))
    }
}

enum FutOrValue<T: Future<Output = Result<Value, crate::Error>>> {
    Fut(T),
    Value(Result<Value, ExecError>),
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TRespHandlerFunc, TRespHandlerFut>
    MiddlewareLike<TLayerCtx>
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
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send + 'static,
    TNewCtx: Send + 'static,
    THandlerFunc: Fn(MiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<MiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
{
    type State = TState;
    type NewCtx = TNewCtx;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError> {
        let handler = (self.handler)(MiddlewareContext {
            state: (),
            ctx,
            input,
            req,
            // new_ctx: None,
            phantom: PhantomData,
        });

        let f = self.resp_handler.clone(); // TODO: Runtime clone is bad. Avoid this!

        Ok(LayerResult::FutureValueOrStreamOrFutureStream(Box::pin(
            async move {
                let handler = handler.await?;

                Ok(
                    match next
                        .call(handler.ctx, handler.input, handler.req)?
                        .into_value_or_stream()
                        .await?
                    {
                        ValueOrStream::Value(v) => {
                            ValueOrStreamOrFutureStream::Value(f(handler.state, v).await?)
                        }
                        ValueOrStream::Stream(s) => {
                            ValueOrStreamOrFutureStream::Stream(Box::pin(s.then(move |v| {
                                let v = match v {
                                    Ok(v) => FutOrValue::Fut(f(handler.state.clone(), v)),
                                    e => FutOrValue::Value(e),
                                };

                                async move {
                                    match v {
                                        FutOrValue::Fut(fut) => {
                                            fut.await.map_err(ExecError::ErrResolverError)
                                        }
                                        FutOrValue::Value(v) => v,
                                    }
                                }
                            })))
                        }
                    },
                )
            },
        )))
    }
}

// TODO: Middleware functions should be able to be async or sync & return a value or result
