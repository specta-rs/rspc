use std::{borrow::Cow, marker::PhantomData};

use specta::TypeDefs;

use crate::{
    internal::{Procedure, ProcedureStore, UnbuiltProcedureBuilder},
    Config, Router,
};

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

    // TODO: Merge over routers here

    // TODO: Mount Middleware -> Or should middleware stick to `Rspc`?

    // TODO: `key` should be `impl Into<Cow<'static, str>>`
    pub fn procedure(mut self, key: &'static str, procedure: impl IntoProcedure<TCtx>) -> Self {
        self.procedures.push((key, Box::new(procedure)));
        self
    }

    // TODO: Return a Legacy router for now
    pub fn compat(self) -> Router<TCtx, ()> {
        // TODO: Eventually take these as an argument so we can access the plugin store from the parent router -> For this we do this for compat
        let mut queries = ProcedureStore::new("queries"); // TODO: Take in as arg // TODO: Combine query, mutations and subscriptions into a big one here
        let mut typ_store = TypeDefs::new(); // TODO: Take in as arg

        let mut ctx = IntoProcedureCtx {
            ty_store: &mut typ_store,
            queries: &mut queries,
        };

        for (key, mut procedure) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            procedure.build(Cow::Borrowed(key), &mut ctx);
        }

        Router {
            config: Config::new(), // TODO: We need to expose this in the new syntax so the user can change it. Can we tak this in at build time not init time?
            queries,
            mutations: ProcedureStore::new("mutations"),
            subscriptions: ProcedureStore::new("subscriptions"),
            typ_store,
            phantom: PhantomData,
        }
    }
}

pub struct IntoProcedureCtx<'a, TCtx> {
    pub ty_store: &'a mut TypeDefs,
    pub queries: &'a mut ProcedureStore<TCtx>,
}

pub trait IntoProcedure<TCtx>: 'static {
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TCtx>);
}
