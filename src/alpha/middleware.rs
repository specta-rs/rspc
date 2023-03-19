use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use specta::{DataType, Type};
use std::future::Future;

use crate::{
    internal::{
        Layer, LayerResult, ProcedureDataType, RequestContext, ValueOrStream,
        ValueOrStreamOrFutureStream,
    },
    ExecError, FutOrValue,
};

use super::AlphaLayer;

// TODO: Rename
pub trait Mw<TLayerCtx>:
    Fn(AlphaMiddlewareBuilder<TLayerCtx>) -> Self::NewMiddleware + Send
where
    TLayerCtx: Send,
{
    type NewLayerCtx: Send;
    type MiddlewareArgMapper: MiddlewareArgMapper;

    type NewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = Self::NewLayerCtx>
        + Send
        + Sync
        + 'static;
}

impl<TLayerCtx, TNewLayerCtx, TNewMiddleware, F> Mw<TLayerCtx> for F
where
    TLayerCtx: Send + Sync,
    TNewLayerCtx: Send,
    TNewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    F: Fn(AlphaMiddlewareBuilder<TLayerCtx>) -> TNewMiddleware + Send,
{
    type NewLayerCtx = TNewLayerCtx;
    type MiddlewareArgMapper = MiddlewareArgMapperPassthrough; // TODO: User defined
    type NewMiddleware = TNewMiddleware;
}

pub trait MiddlewareArgMapper {
    type Input<T>: DeserializeOwned + Type
    where
        T: DeserializeOwned + Type;

    type Output<T>;
}

pub struct MiddlewareArgMapperPassthrough;

impl MiddlewareArgMapper for MiddlewareArgMapperPassthrough {
    type Input<T> = T
    where
        T: DeserializeOwned + Type;
    type Output<T> = T;
}

//
// All of the following stuff is clones of the legacy API with breaking changes for the new system.
//

pub trait AlphaMiddlewareLike<TLayerCtx>: Clone {
    type State: Clone + Send + Sync + 'static;
    type NewCtx: Send + 'static;

    fn handle<TMiddleware: AlphaLayer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
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

    #[cfg(feature = "alpha")] // TODO: Stablise
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

impl<TLayerCtx, TNewCtx, TState> AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>
where
    TLayerCtx: Send,
    TNewCtx: Send,
    TState: Send,
{
    pub fn map_arg(
        self,
        // arg: impl FnOnce(TLayerCtx) -> TNewCtx,
    ) -> AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState> {
        AlphaMiddlewareContext {
            state: self.state,
            input: self.input,
            ctx: self.ctx,
            req: self.req,
            phantom: PhantomData,
        }
    }
}

pub struct AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    handler: THandlerFunc,
    phantom: PhantomData<(TState, TLayerCtx)>,
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut> Clone
    for AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
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

pub struct AlphaMiddlewareBuilder<TLayerCtx>(pub PhantomData<TLayerCtx>)
where
    TLayerCtx: Send;

impl<TLayerCtx> AlphaMiddlewareBuilder<TLayerCtx>
where
    TLayerCtx: Send,
{
    pub fn middleware<TState, TNewCtx, THandlerFunc, THandlerFut>(
        &self,
        handler: THandlerFunc,
    ) -> AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
    where
        TState: Send,
        THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
        THandlerFut: Future<
                Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>,
            > + Send
            + 'static,
    {
        AlphaMiddleware {
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
    AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    pub fn resp<TRespHandlerFunc, TRespHandlerFut>(
        self,
        handler: TRespHandlerFunc,
    ) -> AlphaMiddlewareWithResponseHandler<
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
        AlphaMiddlewareWithResponseHandler {
            handler: self.handler,
            resp_handler: handler,
            phantom: PhantomData,
        }
    }
}

pub struct AlphaMiddlewareWithResponseHandler<
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
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
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
    for AlphaMiddlewareWithResponseHandler<
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
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
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

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut> AlphaMiddlewareLike<TLayerCtx>
    for AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut>
where
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send,
    TNewCtx: Send + 'static,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
{
    type State = TState;
    type NewCtx = TNewCtx;

    fn handle<TMiddleware: AlphaLayer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError> {
        let handler = (self.handler)(AlphaMiddlewareContext {
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

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TRespHandlerFunc, TRespHandlerFut>
    AlphaMiddlewareLike<TLayerCtx>
    for AlphaMiddlewareWithResponseHandler<
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
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>) -> THandlerFut + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
{
    type State = TState;
    type NewCtx = TNewCtx;

    fn handle<TMiddleware: AlphaLayer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError> {
        let handler = (self.handler)(AlphaMiddlewareContext {
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

///
/// `procedure_store.rs`
///

// TODO: Make private
pub struct AlphaProcedure<TCtx> {
    pub exec: Box<dyn AlphaLayer<TCtx>>,
    pub ty: ProcedureDataType,
}

pub struct AlphaProcedureStore<TCtx> {
    name: &'static str,
    pub store: BTreeMap<String, AlphaProcedure<TCtx>>,
}

impl<TCtx> AlphaProcedureStore<TCtx> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            store: BTreeMap::new(),
        }
    }

    pub fn append(&mut self, key: String, exec: Box<dyn AlphaLayer<TCtx>>, ty: ProcedureDataType) {
        #[allow(clippy::panic)]
        if key.is_empty() || key == "ws" || key.starts_with("rpc.") || key.starts_with("rspc.") {
            panic!(
                "rspc error: attempted to create {} operation named '{}', however this name is not allowed.",
                self.name,
                key
            );
        }

        #[allow(clippy::panic)]
        if self.store.contains_key(&key) {
            panic!(
                "rspc error: {} operation already has resolver with name '{}'",
                self.name, key
            );
        }

        self.store.insert(key, AlphaProcedure { exec, ty });
    }
}
