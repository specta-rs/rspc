use std::borrow::Cow;

use specta::TypeDefs;

use crate::internal::{middleware::ProcedureKind, ProcedureStore};

/// TODO
pub(crate) struct BuildProceduresCtx<'a, TCtx> {
    pub(crate) ty_store: &'a mut TypeDefs,
    pub(crate) queries: &'a mut ProcedureStore<TCtx>,
    pub(crate) mutations: &'a mut ProcedureStore<TCtx>,
    pub(crate) subscriptions: &'a mut ProcedureStore<TCtx>,
}

impl<'a, TCtx> BuildProceduresCtx<'a, TCtx> {
    pub fn get_mut(&mut self, kind: ProcedureKind) -> &mut ProcedureStore<TCtx> {
        match kind {
            ProcedureKind::Query => self.queries,
            ProcedureKind::Mutation => self.mutations,
            ProcedureKind::Subscription => self.subscriptions,
        }
    }
}

/// TODO: This exists for the delayed `build` so router context works with merging.
pub(crate) trait DynProcedure<TCtx>: 'static {
    // build takes `&self` but it's safe to assume it will only be run once. It can't take `self` due to dyn Trait restrictions.
    // Due to this prefer `Option::take` instead of `Arc::new` in this method!
    fn build<'b>(&'b mut self, key: Cow<'static, str>, ctx: &'b mut BuildProceduresCtx<'_, TCtx>);
}
