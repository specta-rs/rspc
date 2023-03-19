use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

// TODO: Rename
pub trait Mw<TLayerCtx, TPrevMwMapper, TMwMapper>:
    Fn(AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, ()>) -> Self::NewMiddleware + Send
where
    TLayerCtx: Send,
    TPrevMwMapper: MiddlewareArgMapper,
    TMwMapper: MiddlewareArgMapper,
{
    type NewLayerCtx: Send;
    type NewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = Self::NewLayerCtx, MwMapper = TMwMapper>
        + Send
        + Sync
        + 'static;
}

impl<TLayerCtx, TNewLayerCtx, TNewMiddleware, F, TPrevMwMapper, TMwMapper>
    Mw<TLayerCtx, TPrevMwMapper, TMwMapper> for F
where
    TLayerCtx: Send + Sync,
    TNewLayerCtx: Send,
    TNewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx, MwMapper = TMwMapper>
        + Send
        + Sync
        + 'static,
    F: Fn(AlphaMiddlewareBuilder<TLayerCtx, TPrevMwMapper, ()>) -> TNewMiddleware + Send,
    TPrevMwMapper: MiddlewareArgMapper,
    TMwMapper: MiddlewareArgMapper,
{
    type NewLayerCtx = TNewLayerCtx;
    type NewMiddleware = TNewMiddleware;
}

pub trait MiddlewareArgMapper {
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    type Output<T>: Serialize
    where
        T: Serialize;
    type State;

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

pub trait AlphaMiddlewareLike<TLayerCtx>: Clone {
    type State: Clone + Send + Sync + 'static;
    type NewCtx: Send + 'static;
    type MwMapper: MiddlewareArgMapper;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
        next: Arc<TMiddleware>,
    ) -> Result<LayerResult, ExecError>;
}

pub struct AlphaMiddlewareContext<TLayerCtx, TNewCtx = TLayerCtx, TState = (), TChildArg = ()>
where
    TState: Send,
{
    pub state: TState,
    pub input: Value,
    pub ctx: TNewCtx,
    pub req: RequestContext,
    pub phantom: PhantomData<(TLayerCtx, TChildArg)>,
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

// impl<TLayerCtx, TNewCtx, TState, TChildArg>
//     AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState, TChildArg>
// where
//     TLayerCtx: Send,
//     TNewCtx: Send,
//     TState: Send,
//     TChildArg: Type + DeserializeOwned + 'static,
// {
//     pub fn map_arg<TMapper>(
//         self,
//         arg: impl FnOnce(TMapper::Output<TChildArg>) -> TMapper::Input<TChildArg>,
//     ) -> AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>
//     where
//         TMapper: MiddlewareArgMapper,
//     {
//         // TODO: TMapper

//         AlphaMiddlewareContext {
//             state: self.state,
//             input: self.input,
//             ctx: self.ctx,
//             req: self.req,
//             phantom: PhantomData,
//         }
//     }
// }

pub struct AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
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
    handler: THandlerFunc,
    phantom: PhantomData<(TState, TLayerCtx, TMwMapper)>,
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper> Clone
    for AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
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
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            phantom: PhantomData,
        }
    }
}

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
    // #[cfg(feature = "alpha")] // TODO: Stablise
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
    ) -> AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
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
            phantom: PhantomData,
        }
    }
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
    AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
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
    ) -> AlphaMiddlewareWithResponseHandler<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TRespHandlerFunc,
        TRespHandlerFut,
        TMwMapper,
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
    TMwMapper,
> where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    handler: THandlerFunc,
    resp_handler: TRespHandlerFunc,
    phantom: PhantomData<(TState, TLayerCtx, TMwMapper)>,
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
    > Clone
    for AlphaMiddlewareWithResponseHandler<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TRespHandlerFunc,
        TRespHandlerFut,
        TMwMapper,
    >
where
    TState: Send,
    TLayerCtx: Send,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            resp_handler: self.resp_handler.clone(),
            phantom: PhantomData,
        }
    }
}

impl<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
    AlphaMiddlewareLike<TLayerCtx>
    for AlphaMiddleware<TState, TLayerCtx, TNewCtx, THandlerFunc, THandlerFut, TMwMapper>
where
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send,
    TNewCtx: Send + 'static,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    type State = TState;
    type NewCtx = TNewCtx;
    type MwMapper = TMwMapper;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
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
            next.call(handler.ctx, handler.input, handler.req)?
                .into_value_or_stream()
                .await
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
    > AlphaMiddlewareLike<TLayerCtx>
    for AlphaMiddlewareWithResponseHandler<
        TState,
        TLayerCtx,
        TNewCtx,
        THandlerFunc,
        THandlerFut,
        TRespHandlerFunc,
        TRespHandlerFut,
        TMwMapper,
    >
where
    TState: Clone + Send + Sync + 'static,
    TLayerCtx: Send + 'static,
    TNewCtx: Send + 'static,
    THandlerFunc: Fn(AlphaMiddlewareContext<TLayerCtx, TLayerCtx, ()>, TMwMapper::State) -> THandlerFut
        + Clone,
    THandlerFut: Future<Output = Result<AlphaMiddlewareContext<TLayerCtx, TNewCtx, TState>, crate::Error>>
        + Send
        + 'static,
    TRespHandlerFunc: Fn(TState, Value) -> TRespHandlerFut + Clone + Sync + Send + 'static,
    TRespHandlerFut: Future<Output = Result<Value, crate::Error>> + Send + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    type State = TState;
    type NewCtx = TNewCtx;
    type MwMapper = TMwMapper;

    fn handle<TMiddleware: Layer<Self::NewCtx> + 'static>(
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
    pub exec: Box<dyn Layer<TCtx>>,
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

    pub fn append(&mut self, key: String, exec: Box<dyn Layer<TCtx>>, ty: ProcedureDataType) {
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
