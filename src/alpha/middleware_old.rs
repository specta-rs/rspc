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
