use std::{borrow::Cow, marker::PhantomData, sync::Arc};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    impl_procedure_like,
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareBuilderLike, MiddlewareLayerBuilder,
        ProcedureKind, ResolverLayer, UnbuiltProcedureBuilder,
    },
    typedef, ExecError, MiddlewareBuilder, MiddlewareLike, RequestLayer, SerializeMarker,
};

use super::{IntoProcedure, IntoProcedureCtx};

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
    TMiddleware: MiddlewareBuilderLike<TCtx> + Send + 'static;

impl<TCtx, TLayerCtx, R, RMarker, TMeta>
    AlphaProcedure<TCtx, TLayerCtx, R, RMarker, TMeta, BaseMiddleware<TCtx>>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    R: ResolverFunction<TLayerCtx, RMarker>,
{
    pub(crate) fn new_from_resolver(k: ProcedureKind, resolver: R) -> Self {
        Self(
            Some(resolver),
            BaseMiddleware::default(),
            Some(k),
            PhantomData,
        )
    }
}

impl<TCtx, TLayerCtx, TMeta, TMiddleware>
    AlphaProcedure<TCtx, TLayerCtx, MissingResolver<TLayerCtx>, (), TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    pub(crate) fn new_from_middleware(mw: TMiddleware) -> Self {
        Self(Some(MissingResolver::default()), mw, None, PhantomData)
    }

    impl_procedure_like!();

    // TODO: Mutation + Subscription
}

impl<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
    AlphaProcedure<TCtx, TLayerCtx, R, RMarker, TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    R: ResolverFunction<TLayerCtx, RMarker>,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    pub fn with<TNewLayerCtx, TNewMiddleware>(
        self,
        builder: impl Fn(MiddlewareBuilder<TCtx>) -> TNewMiddleware, // TODO: Remove builder closure
    ) -> AlphaProcedure<
        TCtx,
        TNewLayerCtx,
        MissingResolver<TNewLayerCtx>,
        (),
        (),
        MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>,
    >
    where
        TNewLayerCtx: Send + Sync + 'static,
        TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    {
        let mw = builder(MiddlewareBuilder(PhantomData));
        AlphaProcedure::new_from_middleware(MiddlewareLayerBuilder {
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
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TCtx>) {
        let resolver = Arc::new(self.0.take().expect("Called '.build()' multiple times!")); // TODO: Removing `Arc`?

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
            self.1.build(ResolverLayer {
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
                R::Arg,
                <<R as ResolverFunction<TLayerCtx, RMarker>>::Result as RequestLayer<
                    R::ResultMarker,
                >>::Result,
            >(ctx.ty_store),
        );
    }
}
