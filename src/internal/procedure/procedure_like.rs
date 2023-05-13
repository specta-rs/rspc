#[doc(hidden)]
pub trait IntoProcedureLike<TCtx>: private::SealedIntoProcedureLike<TCtx> {}

mod private {
    use std::borrow::Cow;

    use specta::TypeDefs;

    use crate::internal::{middleware::ProcedureKind, ProcedureStore};

    use super::IntoProcedureLike;

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

    pub trait SealedIntoProcedureLike<TCtx>: 'static {
        // build takes `&self` but it's safe to assume it will only be run once. It can't take `self` due to dyn Trait restrictions.
        // Due to this prefer `Option::take` instead of `Arc::new` in this method!
        fn build<'a, 'b>(
            &'b mut self,
            key: Cow<'static, str>,
            ctx: &'b mut BuildProceduresCtx<'a, TCtx>,
        );
    }

    impl<TCtx, T: SealedIntoProcedureLike<TCtx> + 'static> IntoProcedureLike<TCtx> for T {}
}

pub(crate) use private::{BuildProceduresCtx, SealedIntoProcedureLike};
