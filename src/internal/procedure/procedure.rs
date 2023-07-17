use std::marker::PhantomData;

use std::borrow::Cow;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder,
            ProcedureKind, ResolverLayer,
        },
        procedure::{BuildProceduresCtx, DynProcedure},
        FutureMarkerType, Marker, ProcedureDataType, RequestLayer, ResolverFunction,
        SealedRequestLayer, StreamMarkerType,
    },
    ExecError,
};

/// TODO: Explain
pub struct MissingResolver<TLCtx>(PhantomData<TLCtx>);

impl<TLCtx> Default for MissingResolver<TLCtx> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

mod private {
    pub struct Procedure<T, TMiddleware> {
        pub(crate) resolver: T,
        pub(crate) mw: TMiddleware,
    }
}

pub(crate) use private::Procedure;

impl<TMiddleware, T> Procedure<T, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn new(resolver: T, mw: TMiddleware) -> Self {
        Self { resolver, mw }
    }

    // pub(crate) fn into_dyn_procedure(self) -> Box<dyn DynProcedure<TMiddleware::Ctx>> {
    //     Box::new(self)
    // }
}

// Can only set the resolver or add middleware until a resolver has been set.
// Eg. `.query().subscription()` makes no sense.
impl<TMiddleware> Procedure<MissingResolver<TMiddleware::LayerCtx>, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub fn query<R, RMarker>(
        self,
        resolver: R,
    ) -> Procedure<RMarker, BaseMiddleware<TMiddleware::LayerCtx>>
    where
        R: ResolverFunction<TMiddleware::LayerCtx, RMarker>,
        R::Result: RequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new(
            resolver.into_marker(ProcedureKind::Query),
            BaseMiddleware::default(),
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        resolver: R,
    ) -> Procedure<RMarker, BaseMiddleware<TMiddleware::LayerCtx>>
    where
        R: ResolverFunction<TMiddleware::LayerCtx, RMarker>,
        R::Result: RequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new(
            resolver.into_marker(ProcedureKind::Mutation),
            BaseMiddleware::default(),
        )
    }

    pub fn subscription<R, RMarker>(
        self,
        resolver: R,
    ) -> Procedure<RMarker, BaseMiddleware<TMiddleware::LayerCtx>>
    where
        R: ResolverFunction<TMiddleware::LayerCtx, RMarker>,
        R::Result: RequestLayer<R::RequestMarker, Type = StreamMarkerType>,
    {
        Procedure::new(
            resolver.into_marker(ProcedureKind::Subscription),
            BaseMiddleware::default(),
        )
    }

    pub fn with<Mw: ConstrainedMiddleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<Mw::NewCtx>, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }
}

impl<F, TArg, TResult, TResultMarker, TMiddleware> DynProcedure<TMiddleware::Ctx>
    for Procedure<Marker<F, TMiddleware::LayerCtx, TArg, TResult, TResultMarker>, TMiddleware>
where
    F: Fn(TMiddleware::LayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: Type + DeserializeOwned + 'static,
    TResult: RequestLayer<TResultMarker> + 'static,
    TResultMarker: 'static,
    TMiddleware: MiddlewareBuilder,
{
    fn build<'b>(
        self,
        key: Cow<'static, str>,
        ctx: &'b mut BuildProceduresCtx<'_, TMiddleware::Ctx>,
    ) {
        let Marker(resolver, kind, _) = self.resolver;

        let m = match kind {
            ProcedureKind::Query => &mut ctx.queries,
            ProcedureKind::Mutation => &mut ctx.mutations,
            ProcedureKind::Subscription => &mut ctx.subscriptions,
        };

        let key_str = key.to_string();
        let type_def = ProcedureDataType::from_tys::<TMiddleware::Arg<TArg>, TResult::Result>(
            key,
            ctx.ty_store,
        )
        .expect("error exporting types"); // TODO: Error handling using `#[track_caller]`

        m.append(
            key_str,
            self.mw.build(ResolverLayer::new(move |ctx, input, _| {
                Ok((resolver)(
                    ctx,
                    serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
                )
                .exec())
            })),
            type_def,
        );
    }
}
