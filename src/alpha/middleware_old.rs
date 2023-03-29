use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use specta::{DataType, Type};
use std::future::Future;

use crate::{
    internal::{
        LayerResult, ProcedureDataType, RequestContext, ValueOrStream, ValueOrStreamOrFutureStream,
    },
    ExecError, FutOrValue,
};

use super::AlphaLayer;

// TODO: Remove `LayerResult` from this file

// TODO: Rename
pub trait Mw<TLayerCtx, TPrevMwMapper>:
    Fn(AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, ()>) -> Self::NewMiddleware
where
    TLayerCtx: Send,
    TPrevMwMapper: MiddlewareArgMapper,
{
    type LayerCtx: Send + Sync + 'static;
    type PrevMwMapper: MiddlewareArgMapper;
    type NewLayerCtx: Send;
    type NewMiddleware: AlphaMiddlewareLike<LayerCtx = TLayerCtx, NewCtx = Self::NewLayerCtx>;
}

impl<TLayerCtx, TNewLayerCtx, TNewMiddleware, F, TPrevMwMapper> Mw<TLayerCtx, TPrevMwMapper> for F
where
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send,
    TNewMiddleware: AlphaMiddlewareLike<LayerCtx = TLayerCtx, NewCtx = TNewLayerCtx>,
    F: Fn(AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, ()>) -> TNewMiddleware,
    TPrevMwMapper: MiddlewareArgMapper,
{
    type LayerCtx = TLayerCtx;
    type PrevMwMapper = TPrevMwMapper;
    type NewLayerCtx = TNewMiddleware::NewCtx;
    type NewMiddleware = TNewMiddleware;
}

pub trait MiddlewareArgMapper: Send + Sync {
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    type Output<T>: Serialize
    where
        T: Serialize;
    type State: Send + 'static;

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State);
}

pub struct MiddlewareArgMapperPassthrough;

impl MiddlewareArgMapper for MiddlewareArgMapperPassthrough {
    type Input<T> = T
    where
        T: DeserializeOwned + Type + 'static;
    type Output<T> = T where T: Serialize;
    type State = ();

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State) {
        (arg, ())
    }
}

//
// All of the following stuff is clones of the legacy API with breaking changes for the new system.
//

pub trait AlphaMiddlewareLike: Clone + Send + Sync + 'static {
    type LayerCtx: Send + Sync + 'static;
    type State: Clone + Send + Sync + 'static;
    type NewCtx: Send + Sync + 'static;
    type MwMapper: MiddlewareArgMapper;

    fn handle<TMiddleware: AlphaLayer<Self::NewCtx> + 'static>(
        &self,
        ctx: Self::LayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError>;
}

pub struct AlphaMiddlewareContext<TLayerCtx, TNewCtx = TLayerCtx, TState = ()>
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
impl<TLayerCtx, TNewCtx> AlphaMiddlewareContext<TLayerCtx, TNewCtx, ()>
where
    TLayerCtx: Send,
{
    pub fn with_state<TState>(
        self,
        state: TState,
    ) -> AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>
    where
        TState: Send,
    {
        AlphaMiddlewareContext {
            state,
            input: self.input,
            ctx: self.ctx,
            req: self.req,
            phantom: PhantomData,
        }
    }
}

// This will match were TNewCtx is the default (`TCtx`) so it shouldn't let you call it if you've already swapped the generic
impl<TLayerCtx, TState> AlphaMiddlewareContext<TLayerCtx, TLayerCtx, TState>
where
    TLayerCtx: Send,
    TState: Send,
{
    pub fn with_ctx<TNewCtx>(
        self,
        new_ctx: TNewCtx,
    ) -> AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState> {
        AlphaMiddlewareContext {
            state: self.state,
            input: self.input,
            ctx: new_ctx,
            req: self.req,
            phantom: PhantomData,
        }
    }

    pub fn map_ctx<TNewCtx>(
        self,
        new_ctx: impl FnOnce(TLayerCtx) -> TNewCtx,
    ) -> AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState> {
        AlphaMiddlewareContext {
            state: self.state,
            input: self.input,
            ctx: new_ctx(self.ctx),
            req: self.req,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct MiddlewareNoResponseMarker;
#[derive(Clone)]
pub struct MiddlewareResponseMarker<THandlerFunc>(THandlerFunc);

pub struct AlphaMiddleware<
    TState,
    TLayerCtx,
    TNewCtx,
    THandlerFunc,
    THandlerFut,
    TMwMapper,
    TResponseMarker,
> where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    handler: THandlerFunc,
    resp_handler: TResponseMarker,
    phantom: PhantomData<(TState, TLayerCtx, TMwMapper)>,
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper, TResponseMarker> Clone
    for AlphaMiddleware<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TMwMapper,
        TResponseMarker,
    >
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TMwMapper: MiddlewareArgMapper,
    TResponseMarker: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            resp_handler: self.resp_handler.clone(),
            phantom: PhantomData,
        }
    }
}

// TODO: Remove this
impl MiddlewareArgMapper for () {
    type State = ();
    type Output<T> = T where T: Serialize;
    type Input<T> = T
    where
        T: DeserializeOwned + Type + 'static;

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State) {
        (arg, ())
    }
}

pub struct AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, TMwMapper>(
    pub PhantomData<(TLayerCtx, TPrevMwMapper, TMwMapper)>,
)
where
    TLayerCtx: Send,
    TPrevMwMapper: MiddlewareArgMapper,
    TMwMapper: MiddlewareArgMapper;

impl<TLayerCtx, TPrevMwMapper> AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, ()>
where
    TLayerCtx: Send,
    TPrevMwMapper: MiddlewareArgMapper,
{
    pub fn args<TMiddlewareMapper: MiddlewareArgMapper>(
        &self,
    ) -> AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, TMiddlewareMapper> {
        AlphaMiddlewareBuilder(PhantomData)
    }
}

impl<TLayerCtx, TPrevMwMapper, TMwMapper>
    AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, TMwMapper>
where
    TLayerCtx: Send,
    TPrevMwMapper: MiddlewareArgMapper,
    TMwMapper: MiddlewareArgMapper,
{
    pub fn middleware<TState, TNewCtx, THandlerFunc, THandlerFut>(
        &self,
        handler: THandlerFunc,
    ) -> AlphaMiddleware<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TMwMapper,
        MiddlewareNoResponseMarker,
    >
    where
        TState: Send,
        THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
            + Clone,
        THandlerFut: Future<
                Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>,
            > + Send
            + 'static,
    {
        AlphaMiddleware {
            handler,
            resp_handler: MiddlewareNoResponseMarker,
            phantom: PhantomData,
        }
    }
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
    AlphaMiddleware<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TMwMapper,
        MiddlewareNoResponseMarker,
    >
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    pub fn resp<TRespHandlerFunc, TRespHandlerFut>(
        self,
        handler: TRespHandlerFunc,
    ) -> AlphaMiddleware<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TMwMapper,
        MiddlewareResponseMarker<TRespHandlerFunc>,
    >
    where
        TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
        TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
    {
        AlphaMiddleware {
            handler: self.handler,
            resp_handler: MiddlewareResponseMarker(handler),
            phantom: PhantomData,
        }
    }
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper> AlphaMiddlewareLike
    for AlphaMiddleware<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TMwMapper,
        MiddlewareNoResponseMarker,
    >
where
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewCtx: Send + Sync + 'static,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone
        + Send
        + Sync
        + 'static,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TMwMapper: MiddlewareArgMapper + 'static,
{
    type LayerCtx = TLayerCtx;
    type State = TState;
    type NewCtx = TNewCtx;
    type MwMapper = TMwMapper;

    fn handle<TMiddleware: AlphaLayer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError> {
        let (out, state) =
            TMwMapper::map::<serde_json::Value>(serde_json::from_value(input).unwrap());

        let handler = (self.handler)(
            AlphaMiddlewareContext {
                state: (),
                ctx,
                input: serde_json::to_value(&out).unwrap(),
                req,
                phantom: PhantomData,
            },
            state,
        );

        Ok(LayerResult::FutureValueOrStream(Box::pin(async move {
            let handler = handler.await?;
            next.call(handler.ctx, handler.input, handler.req).await
        })))
    }
}

impl<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TRespHandlerFunc,
        TRespHandlerFut,
        TMwMapper,
    > AlphaMiddlewareLike
    for AlphaMiddleware<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TMwMapper,
        MiddlewareResponseMarker<TRespHandlerFunc>,
    >
where
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewCtx: Send + Sync + 'static,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone
        + Send
        + Sync
        + 'static,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + Sync
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
    TMwMapper: MiddlewareArgMapper + 'static,
{
    type LayerCtx = TLayerCtx;
    type State = TState;
    type NewCtx = TNewCtx;
    type MwMapper = TMwMapper;

    fn handle<TMiddleware: AlphaLayer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError> {
        let (out, state) =
            TMwMapper::map::<serde_json::Value>(serde_json::from_value(input).unwrap());

        let handler = (self.handler)(
            AlphaMiddlewareContext {
                state: (),
                ctx,
                input: serde_json::to_value(&out).unwrap(),
                req,
                // new_ctx: None,
                phantom: PhantomData,
            },
            state,
        );

        let f = self.resp_handler.clone().0; // TODO: Runtime clone is bad. Avoid this!

        Ok(LayerResult::FutureValueOrStreamOrFutureStream(Box::pin(
            async move {
                let handler = handler.await?;

                Ok(
                    match next.call(handler.ctx, handler.input, handler.req).await? {
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
