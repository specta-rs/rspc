mod private {
    use crate::{
        internal::{
            jsonrpc::RequestKind,
            middleware::{
                BaseMiddleware, ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder,
                MissingResolver,
            },
            FutureMarkerType, RequestLayer, RequestLayerMarker, ResolverFunction,
            SealedRequestLayer, StreamLayerMarker, StreamMarkerType,
        },
        ProcedureLike,
    };

    // TODO: `.with` but only support BEFORE resolver is set by the user.

    // TODO: Check metadata stores on this so plugins can extend it to do cool stuff
    // TODO: Logical order for these generics cause right now it's random
    // TODO: Rename `RMarker` so cause we use it at runtime making it not really a "Marker" anymore
    // TODO: Use named struct fields
    pub struct Procedure<R, RMarker, TMiddleware>(
        // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
        Option<R>,
        // Is `None` after `.build()` is called. `.build()` can't take `self` cause dyn safety.
        Option<TMiddleware>,
        RMarker,
    )
    where
        TMiddleware: MiddlewareBuilder;

    impl<TMiddleware, R, RMarker> Procedure<R, RMarker, TMiddleware>
    where
        TMiddleware: MiddlewareBuilder,
    {
        pub fn new_from_resolver(k: RMarker, mw: TMiddleware, resolver: R) -> Self {
            Self(Some(resolver), Some(mw), k)
        }
    }

    impl<TCtx, TLayerCtx> Procedure<MissingResolver<TLayerCtx>, (), BaseMiddleware<TCtx>>
    where
        TCtx: Send + Sync + 'static,
        TLayerCtx: Send + Sync + 'static,
    {
        pub fn new_from_middleware<TMiddleware>(
            mw: TMiddleware,
        ) -> Procedure<MissingResolver<TLayerCtx>, (), TMiddleware>
        where
            TMiddleware: MiddlewareBuilder<Ctx = TCtx>,
        {
            Procedure(Some(MissingResolver::new()), Some(mw), ())
        }
    }

    impl<TMiddleware> Procedure<MissingResolver<TMiddleware::LayerCtx>, (), TMiddleware>
    where
        TMiddleware: MiddlewareBuilder,
    {
        pub fn query<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, RequestLayerMarker<RMarker>, TMiddleware>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Query),
                self.1.take().unwrap(),
                builder,
            )
        }

        pub fn mutation<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, RequestLayerMarker<RMarker>, TMiddleware>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Mutation),
                self.1.take().unwrap(),
                builder,
            )
        }

        pub fn subscription<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, StreamLayerMarker<RMarker>, TMiddleware>
        where
            R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>,
        {
            Procedure::new_from_resolver(StreamLayerMarker::new(), self.1.take().unwrap(), builder)
        }
    }

    impl<TMiddleware> Procedure<MissingResolver<TMiddleware::LayerCtx>, (), TMiddleware>
    where
        TMiddleware: MiddlewareBuilder + Sync,
    {
        pub fn with<Mw: ConstrainedMiddleware<TMiddleware::LayerCtx>>(
            self,
            mw: Mw,
        ) -> Procedure<MissingResolver<Mw::NewCtx>, (), MiddlewareLayerBuilder<TMiddleware, Mw>>
        {
            Procedure::new_from_middleware(MiddlewareLayerBuilder {
                middleware: self.1.expect("Uh oh, stinky"),
                mw,
            })
        }

        #[cfg(feature = "unstable")]
        pub fn with2<Mw: crate::internal::middleware::Middleware<TMiddleware::LayerCtx>>(
            self,
            mw: Mw,
        ) -> Procedure<MissingResolver<Mw::NewCtx>, (), MiddlewareLayerBuilder<TMiddleware, Mw>>
        {
            Procedure::new_from_middleware(MiddlewareLayerBuilder {
                middleware: self.1.expect("Uh oh, stinky"),
                mw,
            })
        }
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
        for Procedure<MissingResolver<TMiddleware::LayerCtx>, (), TMiddleware>
    where
        TMiddleware: MiddlewareBuilder,
    {
        type Middleware = TMiddleware;
        type LayerCtx = TMiddleware::LayerCtx;

        fn query<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Query),
                self.1.take().unwrap(),
                builder,
            )
        }

        fn mutation<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, RequestLayerMarker<RMarker>, Self::Middleware>
        where
            R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
        {
            Procedure::new_from_resolver(
                RequestLayerMarker::new(RequestKind::Query),
                self.1.take().unwrap(),
                builder,
            )
        }

        fn subscription<R, RMarker>(
            mut self,
            builder: R,
        ) -> Procedure<R, StreamLayerMarker<RMarker>, Self::Middleware>
        where
            R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TMiddleware::LayerCtx>
                + Fn(TMiddleware::LayerCtx, R::Arg) -> R::Result,
            R::Result: RequestLayer<R::RequestMarker>
                + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>,
        {
            Procedure::new_from_resolver(StreamLayerMarker::new(), self.1.take().unwrap(), builder)
        }
    }
}

pub(crate) use private::Procedure;
