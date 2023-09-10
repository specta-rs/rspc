use std::{borrow::Cow, panic::Location};

use specta::TypeMap;

use crate::{
    internal::{
        middleware::{MiddlewareBuilder, ProcedureKind},
        procedure::{is_valid_name, BuildProceduresCtx, Procedure, ProcedureStore},
        resolver::HasResolver,
        Layer,
    },
    BuildError, BuildResult, BuiltRouter,
};

type ProcedureBuildFn<TCtx> = Box<dyn FnOnce(Cow<'static, str>, &mut BuildProceduresCtx<'_, TCtx>)>;

pub struct Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    procedures: Vec<(Cow<'static, str>, ProcedureBuildFn<TCtx>)>,
    errors: Vec<BuildError>,
}

impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    /// Constructs a new `Router`.
    /// Avoid using this directly, use [`Rspc::router`] instead so the types can be inferred.
    pub(crate) fn _internal_new() -> Self {
        Self {
            procedures: Vec::new(),
            errors: Vec::new(),
        }
    }

    // TODO: Get `TError` from `Router`?
    #[track_caller]
    pub fn procedure<F, TMiddleware, TError, M>(
        mut self,
        key: &'static str,
        procedure: Procedure<HasResolver<F, TError, M>, TMiddleware>,
    ) -> Self
    where
        HasResolver<F, TError, M>: Layer<TMiddleware::LayerCtx>,
        TMiddleware: MiddlewareBuilder<Ctx = TCtx>,
        M: 'static,
    {
        if let Some(cause) = is_valid_name(key) {
            self.errors.push(BuildError {
                cause,
                #[cfg(debug_assertions)]
                name: Cow::Borrowed(key),
                #[cfg(debug_assertions)]
                loc: Location::caller(),
            });
        }

        self.procedures.push((
            Cow::Borrowed(key),
            Box::new(move |key, ctx| {
                let key_str = key.to_string();
                let type_def = procedure
                    .resolver
                    .into_procedure_def(key, &mut ctx.ty_store)
                    .expect("error exporting types"); // TODO: Error handling using `#[track_caller]`

                let m = match &procedure.resolver.kind {
                    ProcedureKind::Query => &mut ctx.queries,
                    ProcedureKind::Mutation => &mut ctx.mutations,
                    ProcedureKind::Subscription => &mut ctx.subscriptions,
                };

                let layer = procedure.resolver;

                // // TODO: Do this earlier when constructing `HasResolver` if possible?
                // // Trade runtime performance for reduced monomorphization
                // #[cfg(debug_assertions)]
                // let layer = boxed(layer);

                m.append(key_str, procedure.mw.build(layer), type_def);
            }),
        ));

        self
    }

    #[track_caller]
    #[allow(unused_mut)]
    pub fn merge(mut self, prefix: &'static str, mut r: Router<TCtx>) -> Self {
        if let Some(cause) = is_valid_name(prefix) {
            self.errors.push(BuildError {
                cause,
                #[cfg(debug_assertions)]
                name: Cow::Borrowed(prefix),
                #[cfg(debug_assertions)]
                loc: Location::caller(),
            });
        }

        #[cfg(not(debug_assertions))]
        {
            self.errors.append(&mut r.errors);
        }

        #[cfg(debug_assertions)]
        {
            self.errors.extend(&mut r.errors.into_iter().map(|mut err| {
                err.name = Cow::Owned(format!("{}.{}", prefix, err.name));
                err
            }));
        }

        self.procedures.extend(
            r.procedures
                .into_iter()
                .map(|(name, p)| (Cow::Owned(format!("{}.{}", prefix, name)), p)),
        );

        self
    }

    pub fn build(self) -> BuildResult<TCtx> {
        if !self.errors.is_empty() {
            return BuildResult::Err(self.errors);
        }

        // TODO: Eventually take these as an argument so we can access the plugin store from the parent router -> For this we do this for compat
        let mut queries = ProcedureStore::new("queries"); // TODO: Take in as arg
        let mut mutations = ProcedureStore::new("mutations"); // TODO: Take in as arg
        let mut subscriptions = ProcedureStore::new("subscriptions"); // TODO: Take in as arg
        let mut typ_store = TypeMap::new(); // TODO: Take in as arg

        let mut ctx = BuildProceduresCtx {
            ty_store: &mut typ_store,
            queries: &mut queries,
            mutations: &mut mutations,
            subscriptions: &mut subscriptions,
        };

        for (key, build_fn) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            (build_fn)(key, &mut ctx);
        }

        let router = BuiltRouter {
            queries,
            mutations,
            subscriptions,
            typ_store,
        };

        BuildResult::Ok(router)
    }
}
