use crate::internal::procedure::ProcedureStore;

// TODO: Make this `pub(crate)` instead of sealed.
mod private {
    use super::*;

    /// TODO
    pub struct BuildProceduresCtx<'a, TCtx> {
        pub(crate) ty_store: &'a mut specta::TypeMap,
        pub(crate) queries: &'a mut ProcedureStore<TCtx>,
        pub(crate) mutations: &'a mut ProcedureStore<TCtx>,
        pub(crate) subscriptions: &'a mut ProcedureStore<TCtx>,
    }
}
pub(crate) use private::*;
