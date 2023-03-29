use std::{
    any::type_name,
    borrow::Cow,
    future::Ready,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use pin_project::pin_project;
use serde::de::DeserializeOwned;
use specta::{ts::TsExportError, DefOpts, Type, TypeDefs};

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, LayerResult, MiddlewareLayerBuilder,
        ProcedureDataType, ProcedureKind, RequestContext, ResolverLayer, UnbuiltProcedureBuilder,
    },
    ExecError, MiddlewareBuilder, MiddlewareLike, RequestLayer, SerializeMarker,
    StreamRequestLayer,
};

use super::{
    AlphaLayer, AlphaMiddlewareBuilder, AlphaMiddlewareLike, Demo, Executable, Fut, IntoProcedure,
    IntoProcedureCtx, MiddlewareArgMapper, MissingResolver, Mw, MwV2, MwV2Result, ProcedureLike,
    RequestKind, RequestLayerMarker, ResolverFunction, Ret, StreamLayerMarker,
};

/// This exists solely to make Rust shut up about unconstrained generic types

// TODO: `.with` but only support BEFORE resolver is set by the user.

// TODO: Check metadata stores on this so plugins can extend it to do cool stuff
// TODO: Logical order for these generics cause right now it's random
// TODO: Rename `RMarker` so cause we use it at runtime making it not really a "Marker" anymore
// TODO: Use named struct fields
pub struct AlphaProcedure<R, RMarker, TMiddleware>(
    // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
    Option<R>,
    // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
    Option<TMiddleware>,
    RMarker,
)
where
    TMiddleware: AlphaMiddlewareBuilderLike;

impl<TMiddleware, R, RMarker> AlphaProcedure<R, RMarker, TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
{
    pub fn new_from_resolver(k: RMarker, mw: TMiddleware, resolver: R) -> Self {
        Self(Some(resolver), Some(mw), k)
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
        TMiddleware: AlphaMiddlewareBuilderLike<Ctx = TCtx>,
    {
        AlphaProcedure(Some(MissingResolver::default()), Some(mw), ())
    }
}

impl<TMiddleware> AlphaProcedure<MissingResolver<TMiddleware::LayerCtx>, (), TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
{
    pub fn query<R, RMarker>(
        mut self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, TMiddleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
            + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            self.1.take().unwrap(),
            builder,
        )
    }

    pub fn mutation<R, RMarker>(
        mut self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, TMiddleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
            + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Mutation),
            self.1.take().unwrap(),
            builder,
        )
    }

    pub fn subscription<R, RMarker>(
        mut self,
        builder: R,
    ) -> AlphaProcedure<R, StreamLayerMarker<RMarker>, TMiddleware>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
            + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
        R::Result: StreamRequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(StreamLayerMarker::new(), self.1.take().unwrap(), builder)
    }
}

impl<TMiddleware> AlphaProcedure<MissingResolver<TMiddleware::LayerCtx>, (), TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
{
    // TODO: Fix this
    // pub fn with<TNewMiddleware>(
    //     self,
    //     builder: impl Fn(
    //         AlphaMiddlewareBuilder<TMiddleware::LayerCtx, TMiddleware::MwMapper, ()>,
    //     ) -> TNewMiddleware, // TODO: Remove builder closure
    // ) -> AlphaProcedure<
    //     MissingResolver<TNewMiddleware::NewCtx>,
    //     (),
    //     AlphaMiddlewareLayerBuilder<TMiddleware, TNewMiddleware>,
    // >
    // where
    //     TNewMiddleware: AlphaMiddlewareLike<LayerCtx = TMiddleware::LayerCtx>,
    // {
    //     let mw = builder(AlphaMiddlewareBuilder(PhantomData));
    //     AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
    //         middleware: self.1,
    //         mw,
    //     })
    // }
}

impl<R, RMarker, TMiddleware> IntoProcedure<TMiddleware::Ctx>
    for AlphaProcedure<R, RequestLayerMarker<RMarker>, TMiddleware>
where
    R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>,
    RMarker: 'static,
    R::Result: RequestLayer<R::RequestMarker>,
    TMiddleware: AlphaMiddlewareBuilderLike,
{
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TMiddleware::Ctx>) {
        let resolver = Arc::new(self.0.take().expect("Called '.build()' multiple times!")); // TODO: Removing `Arc`?

        let m = match self.2.kind() {
            RequestKind::Query => &mut ctx.queries,
            RequestKind::Mutation => &mut ctx.mutations,
        };

        // m.append_alpha(
        //     key.to_string(),
        //     self.1.take().unwrap().build(AlphaResolverLayer {
        //         func: move |ctx, input, _| {
        //             resolver
        //                 .exec(
        //                     ctx,
        //                     serde_json::from_value(input)
        //                         .map_err(ExecError::DeserializingArgErr)?,
        //                 )
        //                 .into_layer_result()
        //         },
        //         phantom: PhantomData,
        //     }),
        //     R::typedef(key, ctx.ty_store).unwrap(), // TODO: Error handling using `#[track_caller]`
        // );
        todo!();
    }
}

impl<R, RMarker, TMiddleware> IntoProcedure<TMiddleware::Ctx>
    for AlphaProcedure<R, StreamLayerMarker<RMarker>, TMiddleware>
where
    R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>,
    RMarker: 'static,
    R::Result: StreamRequestLayer<R::RequestMarker>,
    TMiddleware: AlphaMiddlewareBuilderLike,
{
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TMiddleware::Ctx>) {
        let resolver = Arc::new(self.0.take().expect("Called '.build()' multiple times!")); // TODO: Removing `Arc`?

        // ctx.subscriptions.append_alpha(
        //     key.to_string(),
        //     self.1.take().unwrap().build(AlphaResolverLayer {
        //         func: move |ctx, input, _| {
        //             resolver
        //                 .exec(
        //                     ctx,
        //                     serde_json::from_value(input)
        //                         .map_err(ExecError::DeserializingArgErr)?,
        //                 )
        //                 .into_layer_result()
        //         },
        //         phantom: PhantomData,
        //     }),
        //     R::typedef(key, ctx.ty_store).unwrap(), // TODO: Error handling using `#[track_caller]`
        // );
        todo!();
    }
}

// TODO: This only works without a resolver. `ProcedureLike` should work on `AlphaProcedure` without it but just without the `.query()` and `.mutate()` functions.
impl<TMiddleware> ProcedureLike
    for AlphaProcedure<MissingResolver<TMiddleware::LayerCtx>, (), TMiddleware>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
{
    type Middleware = TMiddleware;
    type LayerCtx = TMiddleware::LayerCtx;

    fn query<R, RMarker>(
        mut self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
            + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            self.1.take().unwrap(),
            builder,
        )
    }

    // fn mutation<R, RMarker>(
    //     mut self,
    //     builder: R,
    // ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
    // where
    //     R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
    //         + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
    //     R::Result: RequestLayer<R::RequestMarker>,
    // {
    //     AlphaProcedure::new_from_resolver(
    //         RequestLayerMarker::new(RequestKind::Query),
    //         self.1.take().unwrap(),
    //         builder,
    //     )
    // }

    // fn subscription<R, RMarker>(
    //     mut self,
    //     builder: R,
    // ) -> AlphaProcedure<R, StreamLayerMarker<RMarker>, Self::Middleware>
    // where
    //     R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
    //         + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
    //     R::Result: StreamRequestLayer<R::RequestMarker>,
    // {
    //     AlphaProcedure::new_from_resolver(StreamLayerMarker::new(), self.1.take().unwrap(), builder)
    // }
}

///
/// `internal/middleware.rs`
///
use std::future::Future;

use futures::Stream;
use serde_json::Value;

pub trait AlphaMiddlewareBuilderLike: Send + 'static {
    // TODO: Newer stuff
    // type LayerContext: 'static;
    // type LayerResult<T>: Layer<TCtx>
    // where
    //     T: Layer<Self::LayerContext>;

    // fn build<T>(self, next: T) -> Self::LayerResult<T>
    // where
    //     T: Layer<Self::LayerContext>;

    type Ctx: Send + Sync + 'static;
    type LayerCtx: Send + Sync + 'static;
    type MwMapper: MiddlewareArgMapper;
    type IncomingState: Send + 'static; // TODO: Merge this onto something else or take in as `IncomingMiddleware`?

    fn build<T>(self, next: T) -> Box<dyn AlphaLayer<Self::Ctx>>
    where
        T: AlphaLayer<Self::LayerCtx>;
}

pub struct MwArgMapperMerger<TPrev, TNext>(PhantomData<(TPrev, TNext)>)
where
    TPrev: MiddlewareArgMapper,
    TNext: MiddlewareArgMapper;

impl<TPrev, TNext> MiddlewareArgMapper for MwArgMapperMerger<TPrev, TNext>
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
        todo!() // TODO: Is this unreachable?
    }
}

pub struct AlphaMiddlewareLayerBuilder<TMiddleware, TNewMiddleware, TMarker>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
    TMarker: Send,
    TNewMiddleware: MwV2<TMiddleware::LayerCtx, TMarker>,
{
    pub(crate) middleware: TMiddleware,
    pub(crate) mw: TNewMiddleware,
    pub(crate) phantom: PhantomData<TMarker>,
}

impl<TMiddleware, TNewMiddleware, TMarker> AlphaMiddlewareBuilderLike
    for AlphaMiddlewareLayerBuilder<TMiddleware, TNewMiddleware, TMarker>
where
    TMiddleware: AlphaMiddlewareBuilderLike,
    TMarker: Send + 'static,
    TNewMiddleware: MwV2<TMiddleware::LayerCtx, TMarker>,
{
    type Ctx = TMiddleware::Ctx;
    type LayerCtx = TNewMiddleware::NewCtx;
    type MwMapper =
        MwArgMapperMerger<TMiddleware::MwMapper, <TNewMiddleware::Result as MwV2Result>::MwMapper>;
    type IncomingState = <TMiddleware::MwMapper as MiddlewareArgMapper>::State;

    fn build<T>(self, next: T) -> Box<dyn AlphaLayer<TMiddleware::Ctx>>
    where
        T: AlphaLayer<Self::LayerCtx> + Sync,
    {
        todo!();
        // self.middleware.build(AlphaMiddlewareLayer {
        //     next: Arc::new(next), // TODO: Removing `Arc`
        //     mw: self.mw, // .replace().expect("Can't be built twice!"), // Cleanup error or make this impossible in the type system!
        // })
    }
}

pub struct AlphaMiddlewareLayer<TMiddleware, TNewMiddleware>
where
    TMiddleware: AlphaLayer<TNewMiddleware::NewCtx>,
    TNewMiddleware: AlphaMiddlewareLike,
{
    next: Arc<TMiddleware>, // TODO: Avoid arcing this if possible
    mw: TNewMiddleware,
}

impl<TMiddleware, TNewMiddleware> AlphaLayer<TNewMiddleware::LayerCtx>
    for AlphaMiddlewareLayer<TMiddleware, TNewMiddleware>
where
    TMiddleware: AlphaLayer<TNewMiddleware::NewCtx>,
    TNewMiddleware: AlphaMiddlewareLike,
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
    type LayerCtx = TCtx;
    type MwMapper = ();
    type IncomingState = ();

    fn build<T>(self, next: T) -> Box<dyn AlphaLayer<Self::Ctx>>
    where
        T: AlphaLayer<Self::LayerCtx>,
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

impl<T, TLayerCtx> AlphaLayer<TLayerCtx> for AlphaResolverLayer<TLayerCtx, T>
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
