// TODO: Rename
#[doc(hidden)]
pub trait IntoProcedureLike<TCtx>: private::SealedIntoProcedureLike<TCtx> + 'static {}

mod private {
    use specta::TypeDefs;

    use crate::internal::{ProcedureKind, ProcedureStore};

    pub struct BuildProceduresCtx<'a, TCtx> {
        pub(crate) ty_store: &'a mut TypeDefs,
        pub(crate) queries: &'a mut ProcedureStore<TCtx>,
        pub(crate) mutations: &'a mut ProcedureStore<TCtx>,
        pub(crate) subscriptions: &'a mut ProcedureStore<TCtx>,
    }

    impl<'a, TCtx> BuildProceduresCtx<'a, TCtx> {
        pub fn get_mut<'b>(&'b mut self, kind: ProcedureKind) -> &'b mut ProcedureStore<TCtx> {
            match kind {
                ProcedureKind::Query => &mut self.queries,
                ProcedureKind::Mutation => &mut self.mutations,
                ProcedureKind::Subscription => &mut self.subscriptions,
            }
        }
    }

    pub trait SealedIntoProcedureLike<TCtx> {
        // build takes `&self` but it's safe to assume it will only be run once. It can't take `self` due to dyn Trait restrictions.
        // Due to this prefer `Option::take` instead of `Arc::new` in this method!
        // fn build(&self, ctx: BuildProceduresCtx);
    }

    // impl<R, RMarker, TMiddleware, TCtx> SealedIntoProcedureLike<TCtx>
    //     for Procedure<R, RMarker, TMiddleware>
    // {
    //     // fn build(&self, ctx: BuildProceduresCtx) {
    //     //     todo!(); // TODO
    //     // }
    // }
}

pub(crate) use private::BuildProceduresCtx;
