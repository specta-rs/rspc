use std::{any::type_name, borrow::Cow, marker::PhantomData, pin::Pin, sync::Arc};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    impl_procedure_like,
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, Layer, LayerResult, MiddlewareLayerBuilder,
        ProcedureKind, RequestContext, ResolverLayer, UnbuiltProcedureBuilder,
    },
    typedef, ExecError, MiddlewareBuilder, MiddlewareLike, RequestLayer, SerializeMarker,
};

use super::{
    AlphaMiddlewareBuilder, AlphaMiddlewareLike, IntoProcedure, IntoProcedureCtx,
    MiddlewareArgMapper, Mw, ProcedureLike,
};

/// This exists solely to make Rust shut up about unconstrained generic types
pub struct Marker<A, B, C>(PhantomData<(A, B, C)>);

pub trait ResolverFunction<TLayerCtx, TMarker> {
    type Arg: DeserializeOwned + Type;
    type Result: RequestLayer<Self::ResultMarker>;
    type ResultMarker;

    fn exec(&self, ctx: TLayerCtx, arg: Self::Arg) -> Self::Result;
}

impl<
        TLayerCtx,
        TArg,
        TResult,
        TResultMarker,
        F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    > ResolverFunction<TLayerCtx, Marker<TArg, TResult, TResultMarker>> for F
where
    TArg: DeserializeOwned + Type,
    TResult: RequestLayer<TResultMarker>,
{
    type Arg = TArg;
    type Result = TResult;
    type ResultMarker = TResultMarker;

    fn exec(&self, ctx: TLayerCtx, arg: Self::Arg) -> Self::Result {
        self(ctx, arg)
    }
}

pub struct MissingResolver<TLayerCtx> {
    phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx> Default for MissingResolver<TLayerCtx> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<TLayerCtx> ResolverFunction<TLayerCtx, ()> for MissingResolver<TLayerCtx> {
    type Arg = ();
    type Result = ();
    type ResultMarker = SerializeMarker;

    fn exec(&self, _: TLayerCtx, _: Self::Arg) -> Self::Result {
        unreachable!();
    }
}

// TODO: `.with` but only support BEFORE resolver is set by the user.

// TODO: Check metadata stores on this so plugins can extend it to do cool stuff
// TODO: Logical order for these generics cause right now it's random
pub struct AlphaProcedure<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>(
    // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
    Option<R>,
    TMiddleware,
    Option<ProcedureKind>,
    PhantomData<(TCtx, TLayerCtx, RMarker, TMeta)>,
)
where
    TLayerCtx: Send + Sync + 'static,
    R: ResolverFunction<TLayerCtx, RMarker>,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx> + Send + 'static;

impl<TCtx, TLayerCtx, R, RMarker, TMeta>
    AlphaProcedure<TCtx, TLayerCtx, R, RMarker, TMeta, AlphaBaseMiddleware<TCtx>>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    R: ResolverFunction<TLayerCtx, RMarker>,
{
    pub fn new_from_resolver(k: ProcedureKind, resolver: R) -> Self {
        Self(
            Some(resolver),
            AlphaBaseMiddleware::default(),
            Some(k),
            PhantomData,
        )
    }
}

impl<TCtx, TLayerCtx, TMeta>
    AlphaProcedure<
        TCtx,
        TLayerCtx,
        MissingResolver<TLayerCtx>,
        (),
        TMeta,
        AlphaBaseMiddleware<TCtx>,
    >
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
{
    pub fn new_from_middleware<TMiddleware>(
        mw: TMiddleware,
    ) -> AlphaProcedure<TCtx, TLayerCtx, MissingResolver<TLayerCtx>, (), TMeta, TMiddleware>
    where
        TMiddleware: AlphaMiddlewareBuilderLike<TCtx> + Send + 'static,
    {
        AlphaProcedure(Some(MissingResolver::default()), mw, None, PhantomData)
    }
}

impl<TCtx, TLayerCtx, TMeta, TMiddleware>
    AlphaProcedure<TCtx, TLayerCtx, MissingResolver<TLayerCtx>, (), TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    // impl_procedure_like!(); // TODO: Use this instead of redeclaring below

    pub fn query<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<TCtx, TLayerCtx, R, RMarker, (), TMiddleware>
    where
        R: ResolverFunction<TLayerCtx, RMarker> + Fn(TLayerCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure(
            Some(builder),
            self.1,
            Some(ProcedureKind::Query),
            PhantomData,
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<TCtx, TLayerCtx, R, RMarker, (), TMiddleware>
    where
        R: ResolverFunction<TLayerCtx, RMarker> + Fn(TLayerCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure(
            Some(builder),
            self.1,
            Some(ProcedureKind::Mutation),
            PhantomData,
        )
    }
}

impl<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
    AlphaProcedure<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    R: ResolverFunction<TLayerCtx, RMarker>,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    pub fn with<TNewLayerCtx, TNewMiddleware>(
        self,
        builder: impl Fn(AlphaMiddlewareBuilder<TLayerCtx, TMiddleware::MwMapper, ()>) -> TNewMiddleware, // TODO: Remove builder closure
    ) -> AlphaProcedure<
        TCtx,
        TNewLayerCtx,
        MissingResolver<TNewLayerCtx>,
        (),
        (),
        AlphaMiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>,
    >
    where
        TNewLayerCtx: Send + Sync + 'static,
        TNewMiddleware:
            AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    {
        let mw = builder(AlphaMiddlewareBuilder(PhantomData));
        AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
            middleware: self.1,
            mw,
            phantom: PhantomData,
        })
    }
}

// TODO: Only do this impl when `R` is not `MissingResolver`!!!!!
impl<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware> IntoProcedure<TCtx>
    for AlphaProcedure<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    R: ResolverFunction<TLayerCtx, RMarker> + Send + Sync + 'static,
    RMarker: 'static,
    TMeta: 'static,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TCtx>) {
        let resolver = Arc::new(self.0.take().expect("Called '.build()' multiple times!")); // TODO: Removing `Arc`?

        // serde_json::from_value::<TMiddleware::ArgMap<R::Arg>>();

        let m = match self
            .2
            .as_ref()
            .expect("TODO: Make this case impossible in the type system!")
        {
            ProcedureKind::Query => &mut ctx.queries,
            ProcedureKind::Mutation => &mut ctx.mutations,
            ProcedureKind::Subscription => &mut ctx.subscriptions,
        };

        m.append(
            key.into(),
            self.1.build(AlphaResolverLayer {
                func: move |ctx, input, _| {
                    resolver
                        .exec(
                            ctx,
                            serde_json::from_value(input)
                                .map_err(ExecError::DeserializingArgErr)?,
                        )
                        .into_layer_result()
                },
                phantom: PhantomData,
            }),
            typedef::<
                <TMiddleware::MwMapper as MiddlewareArgMapper>::Input<R::Arg>,
                <<R as ResolverFunction<TLayerCtx, RMarker>>::Result as RequestLayer<
                    R::ResultMarker,
                >>::Result,
            >(ctx.ty_store),
        );
    }
}

// TODO: This only works without a resolver. `ProcedureLike` should work on `AlphaProcedure` without it but just without the `.query()` and `.mutate()` functions.
impl<TCtx, TLayerCtx, TMeta, TMiddleware> ProcedureLike<TCtx, TLayerCtx>
    for AlphaProcedure<TCtx, TLayerCtx, MissingResolver<TLayerCtx>, (), TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TMeta: 'static,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    type Middleware = TMiddleware;

    fn query<R2, R2Marker>(
        &self,
        builder: R2,
    ) -> AlphaProcedure<TCtx, TLayerCtx, R2, R2Marker, (), Self::Middleware>
    where
        R2: ResolverFunction<TLayerCtx, R2Marker> + Fn(TLayerCtx, R2::Arg) -> R2::Result,
    {
        // AlphaProcedure::<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
        // Self::query(self, builder)
        todo!();
    }

    // fn query<R2, R2Marker>(
    //     &self,
    //     builder: R2,
    // ) -> AlphaProcedure<TCtx, Self::LayerCtx, R2, R2Marker, (), Self::Middleware>
    // where
    //     R2: ResolverFunction<Self::LayerCtx, R2Marker> + Fn(Self::LayerCtx, R2::Arg) -> R2::Result,
    // {
    //     // AlphaProcedure::<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
    //     // Self::query(self, builder)
    //     todo!();
    // }

    // fn mutation<R, RMarker>(
    //     &self,
    //     builder: R,
    // ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), BaseMiddleware<TCtx>>
    // where
    //     R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result,
    // {
    //     AlphaProcedure::new_from_resolver(ProcedureKind::Mutation, builder)
    // }
}

///
/// `internal/middleware.rs`
///
use std::future::Future;

use futures::Stream;
use serde_json::Value;

pub trait AlphaMiddlewareBuilderLike<TCtx> {
    type LayerContext: 'static;
    type MwMapper: MiddlewareArgMapper;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext>;
}

pub struct MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: AlphaMiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub middleware2: TIncomingMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx)>,
}

pub struct A<TPrev, TNext>(PhantomData<(TPrev, TNext)>)
where
    TPrev: MiddlewareArgMapper,
    TNext: MiddlewareArgMapper;

impl<TPrev, TNext> MiddlewareArgMapper for A<TPrev, TNext>
where
    TPrev: MiddlewareArgMapper,
    TNext: MiddlewareArgMapper,
{
    type Input<T> = TPrev::Input<TNext::Input<T>>
    where
        T: DeserializeOwned + Type + 'static;

    type Output<T> = TNext::Output<TPrev::Output<T>>
    where
        T: serde::Serialize;

    type State = TNext::State;

    fn map<T: serde::Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State) {
        todo!()
    }
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
    AlphaMiddlewareBuilderLike<TCtx>
    for MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TCtx: 'static,
    TLayerCtx: 'static,
    TNewLayerCtx: 'static,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: AlphaMiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    type LayerContext = TNewLayerCtx;
    type MwMapper = A<TMiddleware::MwMapper, TIncomingMiddleware::MwMapper>;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext>,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct AlphaMiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
    TNewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub mw: TNewMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx, TNewLayerCtx)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware, TMwMapper>
    AlphaMiddlewareBuilderLike<TCtx>
    for AlphaMiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: AlphaMiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
    TNewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx, MwMapper = TMwMapper>
        + Send
        + Sync
        + 'static,
    TMwMapper: MiddlewareArgMapper,
{
    type LayerContext = TNewLayerCtx;
    type MwMapper = A<TMiddleware::MwMapper, TNewMiddleware::MwMapper>;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext> + Sync,
    {
        self.middleware.build(AlphaMiddlewareLayer {
            next: Arc::new(next),
            mw: self.mw.clone(),
            phantom: PhantomData,
        })
    }
}

pub struct AlphaMiddlewareLayer<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TMiddleware: Layer<TNewLayerCtx> + 'static,
    TNewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    next: Arc<TMiddleware>, // TODO: Avoid arcing this if possible
    mw: TNewMiddleware,
    phantom: PhantomData<(TLayerCtx, TNewLayerCtx)>,
}

impl<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware> Layer<TLayerCtx>
    for AlphaMiddlewareLayer<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: Layer<TNewLayerCtx> + Sync + 'static,
    TNewMiddleware: AlphaMiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<LayerResult, ExecError> {
        self.mw.handle(ctx, input, req, self.next.clone())
    }
}

pub struct AlphaBaseMiddleware<TCtx>(PhantomData<TCtx>)
where
    TCtx: 'static;

impl<TCtx> Default for AlphaBaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TCtx> AlphaBaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> AlphaMiddlewareBuilderLike<TCtx> for AlphaBaseMiddleware<TCtx>
where
    TCtx: Send + 'static,
{
    type LayerContext = TCtx;
    type MwMapper = ();

    fn build<T>(&self, next: T) -> Box<dyn Layer<TCtx>>
    where
        T: Layer<Self::LayerContext>,
    {
        Box::new(next)
    }
}

pub struct AlphaResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<LayerResult, ExecError>
        + Send
        + Sync
        + 'static,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx> Layer<TLayerCtx> for AlphaResolverLayer<TLayerCtx, T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<LayerResult, ExecError>
        + Send
        + Sync
        + 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError> {
        (self.func)(a, b, c)
    }
}

// impl<TLayerCtx> Layer<TLayerCtx> for Box<dyn Layer<TLayerCtx> + 'static>
// where
//     TLayerCtx: 'static,
// {
//     fn call(&self, a: TLayerCtx, b: Value, c: RequestContext) -> Result<LayerResult, ExecError> {
//         (**self).call(a, b, c)
//     }
// }

// pub enum ValueOrStream {
//     Value(Value),
//     Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
// }

// pub enum ValueOrStreamOrFutureStream {
//     Value(Value),
//     Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
// }

// pub enum LayerResult {
//     Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
//     Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
//     FutureValueOrStream(Pin<Box<dyn Future<Output = Result<ValueOrStream, ExecError>> + Send>>),
//     FutureValueOrStreamOrFutureStream(
//         Pin<Box<dyn Future<Output = Result<ValueOrStreamOrFutureStream, ExecError>> + Send>>,
//     ),
//     Ready(Result<Value, ExecError>),
// }

// impl LayerResult {
//     pub async fn into_value_or_stream(self) -> Result<ValueOrStream, ExecError> {
//         match self {
//             LayerResult::Stream(stream) => Ok(ValueOrStream::Stream(stream)),
//             LayerResult::Future(fut) => Ok(ValueOrStream::Value(fut.await?)),
//             LayerResult::FutureValueOrStream(fut) => Ok(fut.await?),
//             LayerResult::FutureValueOrStreamOrFutureStream(fut) => Ok(match fut.await? {
//                 ValueOrStreamOrFutureStream::Value(val) => ValueOrStream::Value(val),
//                 ValueOrStreamOrFutureStream::Stream(stream) => ValueOrStream::Stream(stream),
//             }),
//             LayerResult::Ready(res) => Ok(ValueOrStream::Value(res?)),
//         }
//     }
// }
