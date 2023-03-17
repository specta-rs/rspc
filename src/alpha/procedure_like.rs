use crate::internal::{BaseMiddleware, ProcedureKind};

use super::{AlphaProcedure, ResolverFunction};

// TODO: Deal with LayerCtx and context switching
pub trait ProcedureLike<TCtx: Send + Sync + 'static> {
    // TODO: Use the `impl_procedure_like!()` if I can fix the visibility issue

    fn query<R, RMarker>(
        &self,
        builder: R,
    ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure::new_from_resolver(ProcedureKind::Query, builder)
    }

    fn mutation<R, RMarker>(
        &self,
        builder: R,
    ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure::new_from_resolver(ProcedureKind::Mutation, builder)
    }

    // TODO: `.subscription`
}

/// This can be used on a type to allow it to be used without the `ProcedureLike` trait in scope.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_procedure_like {
    () => {
        pub fn query<R, RMarker>(
            &self,
            builder: R,
        ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), BaseMiddleware<TCtx>>
        where
            R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result,
        {
            AlphaProcedure::new_from_resolver(ProcedureKind::Query, builder)
        }

        pub fn mutation<R, RMarker>(
            &self,
            builder: R,
        ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), BaseMiddleware<TCtx>>
        where
            R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result,
        {
            AlphaProcedure::new_from_resolver(ProcedureKind::Mutation, builder)
        }

        // TODO: `.subscription`
    };
}

pub use crate::impl_procedure_like;
