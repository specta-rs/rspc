use std::{borrow::Cow, marker::PhantomData};

use specta::TypeDefs;

use crate::{
    internal::{BaseMiddleware, Procedure, ProcedureKind, ProcedureStore, UnbuiltProcedureBuilder},
    Config, RequestKind, RequestLayer, RequestLayerMarker, Router, RouterBuilder,
    RouterBuilderLike,
};

use super::{procedure::AlphaProcedure, AlphaBaseMiddleware, ResolverFunction};

pub struct AlphaRouter<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    procedures: Vec<(&'static str, Box<dyn IntoProcedure<TCtx>>)>,
    phantom: PhantomData<TCtx>,
}

impl<TCtx> AlphaRouter<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    /// Creates a new `AlphaRouter`.
    /// Avoid using this directly, use `Rspc::router` instead so the types can be inferred.
    pub fn new() -> Self {
        Self {
            procedures: Vec::new(),
            phantom: PhantomData,
        }
    }

    // TODO: `key` should be `impl Into<Cow<'static, str>>`
    pub fn procedure(mut self, key: &'static str, procedure: impl IntoProcedure<TCtx>) -> Self {
        self.procedures.push((key, Box::new(procedure)));
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
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::ResultMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            AlphaBaseMiddleware::new(),
            builder,
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::ResultMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Mutation),
            AlphaBaseMiddleware::new(),
            builder,
        )
    }

    // TODO: `.merge()` function

    // TODO: Return a Legacy router for now
    pub fn compat(self) -> Router<TCtx, ()> {
        // TODO: Eventually take these as an argument so we can access the plugin store from the parent router -> For this we do this for compat
        let mut queries = ProcedureStore::new("queries"); // TODO: Take in as arg
        let mut mutations = ProcedureStore::new("mutations"); // TODO: Take in as arg
        let mut subscriptions = ProcedureStore::new("subscriptions"); // TODO: Take in as arg
        let mut typ_store = TypeDefs::new(); // TODO: Take in as arg

        let mut ctx = IntoProcedureCtx {
            ty_store: &mut typ_store,
            queries: &mut queries,
            mutations: &mut mutations,
            subscriptions: &mut subscriptions,
        };

        for (key, mut procedure) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            procedure.build(Cow::Borrowed(key), &mut ctx);
        }

        Router {
            config: Config::new(), // TODO: We need to expose this in the new syntax so the user can change it. Can we tak this in at build time not init time?
            queries,
            mutations,
            subscriptions,
            typ_store,
            phantom: PhantomData,
        }
    }
}

pub struct IntoProcedureCtx<'a, TCtx> {
    pub ty_store: &'a mut TypeDefs,
    pub queries: &'a mut ProcedureStore<TCtx>,
    pub mutations: &'a mut ProcedureStore<TCtx>,
    pub subscriptions: &'a mut ProcedureStore<TCtx>,
}

pub trait IntoProcedure<TCtx>: 'static {
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TCtx>);
}

impl<TCtx> RouterBuilderLike<TCtx> for AlphaRouter<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    type Meta = ();
    type Middleware = BaseMiddleware<TCtx>;

    fn expose(self) -> RouterBuilder<TCtx, Self::Meta, Self::Middleware> {
        let r = self.compat();
        RouterBuilder {
            config: Config::default(),
            middleware: BaseMiddleware::new(),
            queries: r.queries,
            mutations: r.mutations,
            subscriptions: r.subscriptions,
            typ_store: r.typ_store,
            phantom: PhantomData,
        }
    }
}
