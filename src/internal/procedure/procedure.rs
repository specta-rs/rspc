mod private {
    use std::{borrow::Cow, marker::PhantomData};

    use crate::{
        internal::{
            middleware::{
                BaseMiddleware, ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder,
                MissingResolver, ProcedureKind, ResolverLayer,
            },
            procedure::{BuildProceduresCtx, SealedIntoProcedureLike},
            FutureMarkerType, ProcedureMarkerKind, RequestKind, RequestLayer, RequestLayerMarker,
            ResolverFunction, SealedRequestLayer, StreamLayerMarker, StreamMarkerType,
        },
        ExecError, ProcedureLike,
    };

    // TODO: `.with` but only support BEFORE resolver is set by the user.

    // TODO: Check metadata stores on this so plugins can extend it to do cool stuff
    // TODO: Rename `RMarker` so cause we use it at runtime making it not really a "Marker" anymore
    pub struct Procedure<R, TMiddleware, RMarker>
    where
        TMiddleware: MiddlewareBuilder,
    {
        // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
        resolver: Option<R>,
        // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
        mw: Option<TMiddleware>,
        marker: RMarker,
    }

    impl<TMiddleware, R, RMarker> Procedure<R, TMiddleware, RMarker>
    where
        TMiddleware: MiddlewareBuilder,
    {
        pub(crate) fn new_from_resolver(k: RMarker, mw: TMiddleware, resolver: R) -> Self {
            Self {
                resolver: Some(resolver),
                mw: Some(mw),
                marker: k,
            }
        }
    }

    impl<TCtx, TLayerCtx> Procedure<MissingResolver<TLayerCtx>, BaseMiddleware<TCtx>, ()>
    where
        TCtx: Send + Sync + 'static,
        TLayerCtx: Send + Sync + 'static,
    {
        pub(crate) fn new_from_middleware<TMiddleware>(
            mw: TMiddleware,
        ) -> Procedure<MissingResolver<TLayerCtx>, TMiddleware, ()>
        where
            TMiddleware: MiddlewareBuilder<Ctx = TCtx>,
        {
            Procedure {
                resolver: Some(MissingResolver::new()),
                mw: Some(mw),
                marker: (),
            }
        }
    }

    impl<TMiddleware> Procedure<MissingResolver<TMiddleware::LayerCtx>, TMiddleware, ()>
    where
        TMiddleware: MiddlewareBuilder,
    {
        pub fn query<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, TMiddleware, RequestLayerMarker<RMarker>>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Query),
                self.mw.take().expect("error building query"),
                builder,
            )
        }

        pub fn mutation<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, TMiddleware, RequestLayerMarker<RMarker>>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Mutation),
                self.mw.take().expect("error building mutation"),
                builder,
            )
        }

        pub fn subscription<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, TMiddleware, StreamLayerMarker<RMarker>>
        where
            R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>,
        {
            Procedure::new_from_resolver(
                StreamLayerMarker::new(),
                self.mw.take().expect("error building subscription"),
                builder,
            )
        }
    }

    impl<TMiddleware> Procedure<MissingResolver<TMiddleware::LayerCtx>, TMiddleware, ()>
    where
        TMiddleware: MiddlewareBuilder + Sync,
    {
        pub fn with<Mw: ConstrainedMiddleware<TMiddleware::LayerCtx>>(
            self,
            mw: Mw,
        ) -> Procedure<MissingResolver<Mw::NewCtx>, MiddlewareLayerBuilder<TMiddleware, Mw>, ()>
        {
            Procedure::new_from_middleware(MiddlewareLayerBuilder {
                middleware: self.mw.expect("Uh oh, stinky"),
                mw,
            })
        }

        // #[cfg(feature = "unstable")]
        // pub fn with2<Mw: crate::internal::middleware::Middleware<TMiddleware::LayerCtx>>(
        //     self,
        //     mw: Mw,
        // ) -> Procedure<MissingResolver<Mw::NewCtx>, (), MiddlewareLayerBuilder<TMiddleware, Mw>>
        // {
        //     Procedure::new_from_middleware(MiddlewareLayerBuilder {
        //         middleware: self.mw.expect("Uh oh, stinky"),
        //         mw,
        //     })
        // }
    }

    // TODO: Merge this impl into an existing one?
    // impl<R, M: NotAMarker, TMiddleware> Procedure<R, M, TMiddleware>
    // where
    //     R: ResolverFunction<M, LayerCtx = TMiddleware::LayerCtx>,
    //     R::Result: RequestLayer<R::RequestMarker> + SealedRequestLayer<R::RequestMarker>,
    //     TMiddleware: MiddlewareBuilderLike,
    // {
    //     fn build(
    //         &mut self,
    //         key: Cow<'static, str>,
    //         ctx: &mut BuildProceduresCtx<'_, TMiddleware::Ctx>,
    //     ) {
    //         let resolver = self.0.take().expect("Called '.build()' multiple times!");
    //         let type_def = R::typedef::<TMiddleware>(key.clone().into(), ctx.ty_store).unwrap(); // TODO: Error handling using `#[track_caller]`
    //         ctx.get_mut(self.2.kind()).append(
    //             key.into(),
    //             self.1.take().unwrap().build(ResolverLayer {
    //                 func: move |ctx, input, _| {
    //                     Ok(resolver
    //                         .exec(
    //                             ctx,
    //                             serde_json::from_value(input)
    //                                 .map_err(ExecError::DeserializingArgErr)?,
    //                         )
    //                         .exec())
    //                 },
    //                 phantom: PhantomData,
    //             }),
    //             type_def,
    //         );
    //         todo!();
    //     }
    // }

    // TODO: This only works without a resolver. `ProcedureLike` should work on `Procedure` without it but just without the `.query()` and `.mutate()` functions.
    impl<TMiddleware> ProcedureLike
        for Procedure<MissingResolver<TMiddleware::LayerCtx>, TMiddleware, ()>
    where
        TMiddleware: MiddlewareBuilder,
    {
        type Middleware = TMiddleware;
        type LayerCtx = TMiddleware::LayerCtx;

        fn query<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, Self::Middleware, RequestLayerMarker<RMarker>>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Query),
                self.mw.take().expect("rspc: error building query"),
                builder,
            )
        }

        fn mutation<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, Self::Middleware, RequestLayerMarker<RMarker>>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Query),
                self.mw.take().expect("rspc: error building mutation"),
                builder,
            )
        }

        fn subscription<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, Self::Middleware, StreamLayerMarker<RMarker>>
        where
            R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>,
        {
            Procedure::new_from_resolver(
                StreamLayerMarker::new(),
                self.mw.take().expect("rspc: error building subscription"),
                builder,
            )
        }
    }

    impl<R, RMarker, TMiddleware> SealedIntoProcedureLike<TMiddleware::Ctx>
        for Procedure<R, TMiddleware, RMarker>
    where
        R: ResolverFunction<RMarker, LayerCtx = TMiddleware::LayerCtx>,
        RMarker: ProcedureMarkerKind,
        R::Result: RequestLayer<R::RequestMarker>,
        TMiddleware: MiddlewareBuilder,
    {
        fn build<'b>(
            &'b mut self,
            key: Cow<'static, str>,
            ctx: &'b mut BuildProceduresCtx<'_, TMiddleware::Ctx>,
        ) {
            let resolver = self
                .resolver
                .take()
                .expect("Called 'IntoProcedureLike.build()' multiple times!");

            let m = match self.marker.kind() {
                ProcedureKind::Query => &mut ctx.queries,
                ProcedureKind::Mutation => &mut ctx.mutations,
                ProcedureKind::Subscription => &mut ctx.subscriptions,
            };

            let key_str = key.to_string();
            let type_def =
                R::typedef::<TMiddleware>(key, ctx.ty_store).expect("error exporting types"); // TODO: Error handling using `#[track_caller]`
            m.append(
                key_str,
                self.mw
                    .take()
                    .expect("rspc: procedure was built twice. This is a fatal error.")
                    .build(ResolverLayer {
                        func: move |ctx, input, _| {
                            Ok(resolver
                                .exec(
                                    ctx,
                                    serde_json::from_value(input)
                                        .map_err(ExecError::DeserializingArgErr)?,
                                )
                                .exec())
                        },
                        phantom: PhantomData,
                    }),
                type_def,
            );
        }
    }
}

pub(crate) use private::Procedure;
