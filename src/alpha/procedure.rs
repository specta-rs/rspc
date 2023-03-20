use std::{any::type_name, borrow::Cow, marker::PhantomData, pin::Pin, sync::Arc};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    impl_procedure_like,
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, Layer, LayerResult, MiddlewareLayerBuilder,
        ProcedureKind, RequestContext, ResolverLayer, UnbuiltProcedureBuilder,
    },
    typedef, AnyRequestLayer, ExecError, MiddlewareBuilder, MiddlewareLike, RequestLayer,
    RequestLayerMarker, SerializeMarker, StreamLayerMarker, StreamRequestLayer,
};

use super::{
    AlphaMiddlewareBuilder, AlphaMiddlewareLike, IntoProcedure, IntoProcedureCtx,
    MiddlewareArgMapper, Mw, ProcedureLike,
};

/// This exists solely to make Rust shut up about unconstrained generic types

pub trait ResolverFunction<TMarker> {
    type LayerCtx: Send + Sync + 'static;
    type Arg: DeserializeOwned + Type;
    type Result: AnyRequestLayer<Self::ResultMarker>;
    type ResultMarker;

    fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result;
}

pub struct Marker<A, B, C, D>(PhantomData<(A, B, C, D)>);
impl<
        TLayerCtx,
        TArg,
        TResult,
        TResultMarker,
        F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    > ResolverFunction<Marker<TArg, TResult, TResultMarker, TLayerCtx>> for F
where
    TArg: DeserializeOwned + Type,
    TResult: RequestLayer<TResultMarker>,
    TLayerCtx: Send + Sync + 'static,
{
    type LayerCtx = TLayerCtx;
    type Arg = TArg;
    type Result = TResult;
    type ResultMarker = RequestLayerMarker<TResultMarker>;

    fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result {
        self(ctx, arg)
    }
}

pub struct SubscriptionMarker<A, B, C, D>(PhantomData<(A, B, C, D)>);
impl<
        TLayerCtx,
        TArg,
        TResult,
        TResultMarker,
        F: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    > ResolverFunction<SubscriptionMarker<TArg, TResult, TResultMarker, TLayerCtx>> for F
where
    TArg: DeserializeOwned + Type,
    TResult: StreamRequestLayer<TResultMarker>,
    TLayerCtx: Send + Sync + 'static,
{
    type LayerCtx = TLayerCtx;
    type Arg = TArg;
    type Result = TResult;
    type ResultMarker = StreamLayerMarker<TResultMarker>;

    fn exec(&self, ctx: Self::LayerCtx, arg: Self::Arg) -> Self::Result {
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

impl<TLayerCtx> ResolverFunction<()> for MissingResolver<TLayerCtx>
where
    TLayerCtx: Send + Sync + 'static,
{
    type LayerCtx = TLayerCtx;
    type Arg = ();
    type Result = ();
    type ResultMarker = RequestLayerMarker<SerializeMarker>;

    fn exec(&self, _: Self::LayerCtx, _: Self::Arg) -> Self::Result {
        unreachable!();
    }
}

// TODO: `.with` but only support BEFORE resolver is set by the user.

// TODO: Check metadata stores on this so plugins can extend it to do cool stuff
// TODO: Logical order for these generics cause right now it's random
pub struct AlphaProcedure<R, RMarker, TMiddleware>(
    // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
    Option<R>,
    TMiddleware,
    Option<ProcedureKind>,
    PhantomData<(RMarker)>,
)
where
    R: ResolverFunction<RMarker>,
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static;

impl<TMiddleware, R, RMarker> AlphaProcedure<R, RMarker, TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
    R: ResolverFunction<RMarker>,
{
    pub fn new_from_resolver(k: ProcedureKind, mw: TMiddleware, resolver: R) -> Self {
        Self(Some(resolver), mw, Some(k), PhantomData)
    }
}

impl<TCtx, TLayerCtx> AlphaProcedure<MissingResolver<TLayerCtx>, (), AlphaBaseMiddleware<TCtx>>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
{
    pub fn new_from_middleware<TMiddleware>(
        mw: TMiddleware,
    ) -> AlphaProcedure<MissingResolver<TLayerCtx>, (), TMiddleware>
    where
        TMiddleware: AlphaMiddlewareBuilderLike<Ctx = TCtx> + Send + 'static,
    {
        AlphaProcedure(Some(MissingResolver::default()), mw, None, PhantomData)
    }
}

impl<TMiddleware> AlphaProcedure<MissingResolver<TMiddleware::LayerContext>, (), TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
{
    // impl_procedure_like!(); // TODO: Use this instead of redeclaring below

    pub fn query<R, RMarker>(self, builder: R) -> AlphaProcedure<R, RMarker, TMiddleware>
    where
        R: ResolverFunction<RMarker, LayerCtx = TMiddleware::LayerContext>
            + Fn(TMiddleware::LayerContext, R::Arg) -> R::Result,
    {
        AlphaProcedure(
            Some(builder),
            self.1,
            Some(ProcedureKind::Query),
            PhantomData,
        )
    }

    pub fn mutation<R, RMarker>(self, builder: R) -> AlphaProcedure<R, RMarker, TMiddleware>
    where
        R: ResolverFunction<RMarker, LayerCtx = TMiddleware::LayerContext>
            + Fn(TMiddleware::LayerContext, R::Arg) -> R::Result,
    {
        AlphaProcedure(
            Some(builder),
            self.1,
            Some(ProcedureKind::Mutation),
            PhantomData,
        )
    }

    pub fn subscription<R, RMarker>(self, builder: R) -> AlphaProcedure<R, RMarker, TMiddleware>
    where
        R: ResolverFunction<RMarker, LayerCtx = TMiddleware::LayerContext>
            + Fn(TMiddleware::LayerContext, R::Arg) -> R::Result,
    {
        AlphaProcedure(
            Some(builder),
            self.1,
            Some(ProcedureKind::Subscription),
            PhantomData,
        )
    }
}

impl<R, RMarker, TMiddleware> AlphaProcedure<R, RMarker, TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
    R: ResolverFunction<RMarker, LayerCtx = TMiddleware::LayerContext>,
{
    pub fn with<TNewMiddleware>(
        self,
        builder: impl Fn(
            AlphaMiddlewareBuilder<TMiddleware::LayerContext, TMiddleware::MwMapper, ()>,
        ) -> TNewMiddleware, // TODO: Remove builder closure
    ) -> AlphaProcedure<
        MissingResolver<TNewMiddleware::NewCtx>,
        (),
        AlphaMiddlewareLayerBuilder<TMiddleware, TNewMiddleware>,
    >
    where
        TNewMiddleware:
            AlphaMiddlewareLike<LayerCtx = TMiddleware::LayerContext> + Send + Sync + 'static,
    {
        let mw = builder(AlphaMiddlewareBuilder(PhantomData));
        AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
            middleware: self.1,
            mw,
        })
    }
}

// TODO: Only do this impl when `R` is not `MissingResolver`!!!!!
impl<R, RMarker, TMiddleware> IntoProcedure<TMiddleware::Ctx>
    for AlphaProcedure<R, RMarker, TMiddleware>
where
    R: ResolverFunction<RMarker, LayerCtx = TMiddleware::LayerContext> + Send + Sync + 'static,
    RMarker: 'static,
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
{
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TMiddleware::Ctx>) {
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
                        .any_into_layer_result()
                },
                phantom: PhantomData,
            }),
            typedef::<
                <TMiddleware::MwMapper as MiddlewareArgMapper>::Input<R::Arg>,
                <<R as ResolverFunction<RMarker>>::Result as AnyRequestLayer<R::ResultMarker>>::Result,
            >(ctx.ty_store),
        );
    }
}

// TODO: This only works without a resolver. `ProcedureLike` should work on `AlphaProcedure` without it but just without the `.query()` and `.mutate()` functions.
impl<TMiddleware> ProcedureLike<TMiddleware::LayerContext>
    for AlphaProcedure<MissingResolver<TMiddleware::LayerContext>, (), TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
{
    type Middleware = TMiddleware;

    fn query<R2, R2Marker>(self, builder: R2) -> AlphaProcedure<R2, R2Marker, Self::Middleware>
    where
        R2: ResolverFunction<R2Marker, LayerCtx = TMiddleware::LayerContext>
            + Fn(TMiddleware::LayerContext, R2::Arg) -> R2::Result,
    {
        AlphaProcedure::new_from_resolver(ProcedureKind::Query, self.1, builder)
    }

    // fn query<R2, R2Marker>(
    //     &self,
    //     builder: R2,
    // ) -> AlphaProcedure<TCtx, Self::LayerCtx, R2, R2Marker, (), Self::Middleware>
    // where
    //     R2: ResolverFunction<Self::LayerCtx, R2Marker> + Fn(Self::LayerCtx, R2::Arg) -> R2::Result,
    // {
    //     // AlphaProcedure::<TCtx, TLayerCtx, R, RMarker, TMiddleware>
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

pub trait AlphaMiddlewareBuilderLike {
    type Ctx: Send + Sync + 'static;
    type LayerContext: Send + Sync + 'static;
    type MwMapper: MiddlewareArgMapper;

    fn build<T>(&self, next: T) -> Box<dyn Layer<Self::Ctx>>
    where
        T: Layer<Self::LayerContext>;
}

pub struct MiddlewareMerger<TMiddleware, TIncomingMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
    TIncomingMiddleware: AlphaMiddlewareBuilderLike<Ctx = TMiddleware::LayerContext>,
{
    pub middleware: TMiddleware,
    pub middleware2: TIncomingMiddleware,
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

impl<TMiddleware, TIncomingMiddleware> AlphaMiddlewareBuilderLike
    for MiddlewareMerger<TMiddleware, TIncomingMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
    TIncomingMiddleware: AlphaMiddlewareBuilderLike<Ctx = TMiddleware::LayerContext>,
{
    type Ctx = TMiddleware::Ctx;
    type LayerContext = TIncomingMiddleware::LayerContext;
    type MwMapper = A<TMiddleware::MwMapper, TIncomingMiddleware::MwMapper>;

    fn build<T>(&self, next: T) -> Box<dyn Layer<Self::Ctx>>
    where
        T: Layer<Self::LayerContext>,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct AlphaMiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
    TNewMiddleware: AlphaMiddlewareLike<LayerCtx = TMiddleware::LayerContext>,
{
    pub middleware: TMiddleware,
    pub mw: TNewMiddleware,
}

impl<TMiddleware, TNewMiddleware> AlphaMiddlewareBuilderLike
    for AlphaMiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike + Send + 'static,
    TNewMiddleware:
        AlphaMiddlewareLike<LayerCtx = TMiddleware::LayerContext> + Send + Sync + 'static,
{
    type Ctx = TMiddleware::Ctx;
    type LayerContext = TNewMiddleware::NewCtx;
    type MwMapper = A<TMiddleware::MwMapper, TNewMiddleware::MwMapper>;

    fn build<T>(&self, next: T) -> Box<dyn Layer<TMiddleware::Ctx>>
    where
        T: Layer<Self::LayerContext> + Sync,
    {
        self.middleware.build(AlphaMiddlewareLayer {
            next: Arc::new(next),
            mw: self.mw.clone(),
        })
    }
}

pub struct AlphaMiddlewareLayer<TMiddleware, TNewMiddleware>
where
    TMiddleware: Layer<TNewMiddleware::NewCtx> + 'static,
    TNewMiddleware: AlphaMiddlewareLike + Send + Sync + 'static,
{
    next: Arc<TMiddleware>, // TODO: Avoid arcing this if possible
    mw: TNewMiddleware,
}

impl<TMiddleware, TNewMiddleware> Layer<TNewMiddleware::LayerCtx>
    for AlphaMiddlewareLayer<TMiddleware, TNewMiddleware>
where
    TMiddleware: Layer<TNewMiddleware::NewCtx> + Sync + 'static,
    TNewMiddleware: AlphaMiddlewareLike + Send + Sync + 'static,
{
    fn call(
        &self,
        ctx: TNewMiddleware::LayerCtx,
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

impl<TCtx> AlphaMiddlewareBuilderLike for AlphaBaseMiddleware<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    type Ctx = TCtx;
    type LayerContext = TCtx;
    type MwMapper = ();

    fn build<T>(&self, next: T) -> Box<dyn Layer<Self::Ctx>>
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
