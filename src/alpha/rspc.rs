use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareBuilderLike, MiddlewareLayerBuilder,
        UnbuiltProcedureBuilder,
    },
    MiddlewareBuilder, MiddlewareLike, RequestLayer, RouterBuilder,
};

use super::{AlphaProcedure, AlphaRouter, MissingResolver, ResolverFunction};

pub struct Rspc<
    TCtx = (), // The is the context the current router was initialised with
> where
    TCtx: Send + Sync + 'static,
{
    builders: Vec<Box<dyn FnOnce()>>,
    phantom: PhantomData<TCtx>,
    // queries: ProcedureStore<TCtx>,
    // mutations: ProcedureStore<TCtx>,
    // subscriptions: ProcedureStore<TCtx>,
}

#[allow(clippy::new_without_default)]
impl<TCtx> Rspc<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            builders: Vec::new(),
            phantom: PhantomData,
            // queries: ProcedureStore::new("query"),
            // mutations: ProcedureStore::new("mutation"),
            // subscriptions: ProcedureStore::new("subscription"),
        }
    }
}

impl<TCtx> Rspc<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub fn router(&self) -> AlphaRouter<TCtx> {
        AlphaRouter::new()
    }

    // TODO: Remove the `BaseMiddleware` from this join cause it shouldn't be required
    pub fn with<TNewLayerCtx, TNewMiddleware>(
        &self,
        builder: impl Fn(MiddlewareBuilder<TCtx>) -> TNewMiddleware, // TODO: Remove builder closure
    ) -> AlphaProcedure<
        TCtx,
        TNewLayerCtx,
        MissingResolver<TNewLayerCtx>,
        (),
        (),
        MiddlewareLayerBuilder<TCtx, TCtx, TNewLayerCtx, BaseMiddleware<TCtx>, TNewMiddleware>,
    >
    where
        TNewLayerCtx: Send + Sync + 'static,
        TNewMiddleware: MiddlewareLike<TCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    {
        let mw = builder(MiddlewareBuilder(PhantomData));
        AlphaProcedure::new_from_middleware(MiddlewareLayerBuilder {
            middleware: BaseMiddleware::new(),
            mw,
            phantom: PhantomData,
        })
    }

    // pub fn with<TNewMiddleware, TNewLayerCtx>(
    //     &self,
    // ) -> RouterBuilder<
    //     TCtx,
    //     (),
    //     MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>,
    // >
    // where
    //     TNewLayerCtx: Send + Sync + 'static,
    //     TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    // {
    //     todo!();
    //     // let Self {
    //     //     config,
    //     //     middleware,
    //     //     queries,
    //     //     mutations,
    //     //     subscriptions,
    //     //     typ_store,
    //     //     ..
    //     // } = self;

    //     // let mw = builder(MiddlewareBuilder(PhantomData));
    //     // RouterBuilder {
    //     //     config,
    //     //     middleware: MiddlewareLayerBuilder {
    //     //         middleware,
    //     //         mw,
    //     //         phantom: PhantomData,
    //     //     },
    //     //     queries,
    //     //     mutations,
    //     //     subscriptions,
    //     //     typ_store,
    //     //     phantom: PhantomData,
    //     // }
    // }

    pub fn query<R, RMarker>(
        &self,
        builder: R,
    ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure::new_from_resolver(builder)
    }

    // pub fn query<TResolver, TArg, TResult, TResultMarker, TBuilder>(
    //     &self,
    //     builder: R,
    // ) -> AlphaProcedure<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, ()>
    // where
    //     TArg: DeserializeOwned + Type,
    //     TResult: RequestLayer<TResultMarker>,
    //     TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    //     TBuilder:
    //         Fn(UnbuiltProcedureBuilder<TLayerCtx, TResolver>) -> BuiltProcedureBuilder<TResolver>,
    // {
    //     AlphaProcedure::new_from_resolver(builder)
    // }

    // pub fn mutation<TResolver, TArg, TResult, TResultMarker>(
    //     mut self,
    //     key: &'static str,
    //     builder: impl Fn(
    //         UnbuiltProcedureBuilder<TLayerCtx, TResolver>,
    //     ) -> BuiltProcedureBuilder<TResolver>,
    // ) -> Self
    // where
    //     TArg: DeserializeOwned + Type,
    //     TResult: RequestLayer<TResultMarker>,
    //     TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    // {
    //     let resolver = builder(UnbuiltProcedureBuilder::default()).resolver;
    //     self.mutations.append(
    //         key.into(),
    //         self.middleware.build(ResolverLayer {
    //             func: move |ctx, input, _| {
    //                 resolver.exec(
    //                     ctx,
    //                     serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
    //                 )
    //             },
    //             phantom: PhantomData,
    //         }),
    //         TResolver::typedef(&mut self.typ_store),
    //     );
    //     self
    // }

    // pub fn subscription<TResolver, TArg, TStream, TResult, TResultMarker>(
    //     mut self,
    //     key: &'static str,
    //     builder: impl Fn(
    //         UnbuiltProcedureBuilder<TLayerCtx, TResolver>,
    //     ) -> BuiltProcedureBuilder<TResolver>,
    // ) -> Self
    // where
    //     TArg: DeserializeOwned + Type,
    //     TStream: Stream<Item = TResult> + Send + 'static,
    //     TResult: Serialize + Type,
    //     TResolver: Fn(TLayerCtx, TArg) -> TStream
    //         + StreamResolver<TLayerCtx, DoubleArgStreamMarker<TArg, TResultMarker, TStream>>
    //         + Send
    //         + Sync
    //         + 'static,
    // {
    //     let resolver = builder(UnbuiltProcedureBuilder::default()).resolver;
    //     self.subscriptions.append(
    //         key.into(),
    //         self.middleware.build(ResolverLayer {
    //             func: move |ctx, input, _| {
    //                 resolver.exec(
    //                     ctx,
    //                     serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
    //                 )
    //             },
    //             phantom: PhantomData,
    //         }),
    //         TResolver::typedef(&mut self.typ_store),
    //     );
    //     self
    // }

    // pub fn merge<TNewLayerCtx, TIncomingMiddleware>(
    //     mut self,
    //     prefix: &'static str,
    //     router: RouterBuilder<TLayerCtx, TMeta, TIncomingMiddleware>,
    // ) -> Self
    // where
    //     TNewLayerCtx: 'static,
    //     TIncomingMiddleware:
    //         MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx> + Send + 'static,
    // {
    //     #[allow(clippy::panic)]
    //     if prefix.is_empty() || prefix.starts_with("rpc.") || prefix.starts_with("rspc.") {
    //         panic!(
    //             "rspc error: attempted to merge a router with the prefix '{}', however this name is not allowed.",
    //             prefix
    //         );
    //     }

    //     // TODO: The `data` field has gotta flow from the root router to the leaf routers so that we don't have to merge user defined types.

    //     for (key, query) in router.queries.store {
    //         // query.ty.key = format!("{}{}", prefix, key);
    //         self.queries.append(
    //             format!("{}{}", prefix, key),
    //             self.middleware.build(query.exec),
    //             query.ty,
    //         );
    //     }

    //     for (key, mutation) in router.mutations.store {
    //         // mutation.ty.key = format!("{}{}", prefix, key);
    //         self.mutations.append(
    //             format!("{}{}", prefix, key),
    //             self.middleware.build(mutation.exec),
    //             mutation.ty,
    //         );
    //     }

    //     for (key, subscription) in router.subscriptions.store {
    //         // subscription.ty.key = format!("{}{}", prefix, key);
    //         self.subscriptions.append(
    //             format!("{}{}", prefix, key),
    //             self.middleware.build(subscription.exec),
    //             subscription.ty,
    //         );
    //     }

    //     for (name, typ) in router.typ_store {
    //         self.typ_store.insert(name, typ);
    //     }

    //     self
    // }

    // pub fn build(self) -> Router<TCtx, TMeta> {
    //     let Self {
    //         config,
    //         queries,
    //         mutations,
    //         subscriptions,
    //         typ_store,
    //         ..
    //     } = self;

    //     let export_path = config.export_bindings_on_build.clone();
    //     let router = Router {
    //         config,
    //         queries,
    //         mutations,
    //         subscriptions,
    //         typ_store,
    //         phantom: PhantomData,
    //     };

    //     #[cfg(debug_assertions)]
    //     #[allow(clippy::unwrap_used)]
    //     if let Some(export_path) = export_path {
    //         router.export_ts(export_path).unwrap();
    //     }

    //     router
    // }

    // pub fn delayed_build(self, config: Config) -> () {
    //     // let typ_store: TypeDefs::new();
    //     todo!();
    // }
}
