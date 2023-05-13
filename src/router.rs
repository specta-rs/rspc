use std::{borrow::Cow, marker::PhantomData};

use specta::TypeDefs;

use crate::{
    internal::{
        jsonrpc::RequestKind,
        middleware::BaseMiddleware,
        procedure::{BuildProceduresCtx, IntoProcedureLike, Procedure},
        FutureMarkerType, ProcedureStore, RequestLayer, RequestLayerMarker, ResolverFunction,
        SealedRequestLayer, StreamLayerMarker, StreamMarkerType,
    },
    CompiledRouter, Config,
};

pub struct Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    procedures: Vec<(Cow<'static, str>, Box<dyn IntoProcedureLike<TCtx>>)>,
}

impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    /// Constructs a new `Router`.
    /// Avoid using this directly, use `Rspc::router` instead so the types can be inferred.
    pub(crate) fn new() -> Self {
        Self {
            procedures: Vec::new(),
        }
    }

    pub fn procedure<R, RMarker, TMiddleware>(
        mut self,
        key: &'static str,
        procedure: impl IntoProcedureLike<TCtx>,
    ) -> Self {
        self.procedures
            .push((Cow::Borrowed(key), Box::new(procedure)));
        self
    }

    // TODO
    // pub fn merge(self, prefix: &'static str, r: impl RouterBuilderLike<TCtx>) -> Self {
    //     // TODO: disallow `.` in prefix
    //     let r = r.expose();
    //     todo!();
    // }

    pub fn query<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, RequestLayerMarker<RMarker>, BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::ResultMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            BaseMiddleware::new(),
            builder,
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, RequestLayerMarker<RMarker>, BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::ResultMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Mutation),
            BaseMiddleware::new(),
            builder,
        )
    }

    pub fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> Procedure<R, StreamLayerMarker<RMarker>, BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>,
    {
        Procedure::new_from_resolver(StreamLayerMarker::new(), BaseMiddleware::new(), builder)
    }

    // TODO: Get this working
    // pub fn merge(
    //     mut self,
    //     prefix: &'static str,
    //     router: impl RouterBuilderLike<TCtx>,
    // ) -> Self {
    //     // TODO
    //     // let (prefix, prefix_valid) = is_invalid_router_prefix(prefix);
    //     // #[allow(clippy::panic)]
    //     // if prefix_valid {
    //     //     eprintln!(
    //     //         "{}: rspc error: attempted to merge a router with the prefix '{}', however this prefix is not allowed. ",
    //     //         Location::caller(),
    //     //         prefix
    //     //     );
    //     //     process::exit(1);
    //     // }

    //     self.procedures.extend(
    //         router
    //             .procedures()
    //             .into_iter()
    //             .map(|(key, procedure)| (Cow::Owned(format!("{}{}", prefix, key)), procedure)),
    //     );

    //     self
    // }

    // #[deprecated = "TODO: Remove this"]
    // pub fn compat(self) -> BuiltRouter<TCtx, ()> {
    //     // TODO: Eventually take these as an argument so we can access the plugin store from the parent router -> For this we do this for compat
    //     let mut queries = ProcedureStore::new("queries"); // TODO: Take in as arg
    //     let mut mutations = ProcedureStore::new("mutations"); // TODO: Take in as arg
    //     let mut subscriptions = ProcedureStore::new("subscriptions"); // TODO: Take in as arg
    //     let mut typ_store = TypeDefs::new(); // TODO: Take in as arg

    //     let mut ctx = IntoProceduresCtx {
    //         ty_store: &mut typ_store,
    //         queries: &mut queries,
    //         mutations: &mut mutations,
    //         subscriptions: &mut subscriptions,
    //     };

    //     for (key, mut procedure) in self.procedures.into_iter() {
    //         // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
    //         procedure.build(key, &mut ctx);
    //     }

    //     BuiltRouter {
    //         config: Config::new(),
    //         queries,
    //         mutations,
    //         subscriptions,
    //         typ_store,
    //         phantom: PhantomData,
    //     }
    // }

    // TODO: Change the return type and clean this whole system up
    pub fn build(self, config: Config) -> CompiledRouter<TCtx, ()> {
        // TODO: Eventually take these as an argument so we can access the plugin store from the parent router -> For this we do this for compat
        let mut queries = ProcedureStore::new("queries"); // TODO: Take in as arg
        let mut mutations = ProcedureStore::new("mutations"); // TODO: Take in as arg
        let mut subscriptions = ProcedureStore::new("subscriptions"); // TODO: Take in as arg
        let mut typ_store = TypeDefs::new(); // TODO: Take in as arg

        let mut ctx = BuildProceduresCtx {
            ty_store: &mut typ_store,
            queries: &mut queries,
            mutations: &mut mutations,
            subscriptions: &mut subscriptions,
        };

        for (key, mut procedure) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            // procedure.build(key, &mut ctx);
            todo!(); // TODO
        }

        let router = CompiledRouter {
            config,
            queries,
            mutations,
            subscriptions,
            typ_store,
            phantom: PhantomData,
        };

        #[cfg(debug_assertions)]
        #[allow(clippy::unwrap_used)]
        if let Some(export_path) = &router.config.export_bindings_on_build {
            router.export_ts(export_path).unwrap();
        }

        router
    }
}
