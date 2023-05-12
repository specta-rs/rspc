#[doc(hidden)]
pub trait IntoProcedures<TCtx>: SealedIntoProcedures<TCtx> {}

mod private {
    use std::borrow::Cow;

    use specta::TypeDefs;

    use crate::internal::ProcedureStore;

    pub struct IntoProceduresCtx<'a, TCtx> {
        pub ty_store: &'a mut TypeDefs,
        pub queries: &'a mut ProcedureStore<TCtx>,
        pub mutations: &'a mut ProcedureStore<TCtx>,
        pub subscriptions: &'a mut ProcedureStore<TCtx>,
    }

    pub trait SealedIntoProcedures<TCtx>: 'static {
        fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProceduresCtx<'_, TCtx>);
    }

    impl<TCtx, T: SealedIntoProcedures<TCtx>> super::IntoProcedures<TCtx> for T {}
}

pub(crate) use private::{IntoProceduresCtx, SealedIntoProcedures};
