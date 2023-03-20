use crate::internal::ProcedureKind;

use super::{AlphaMiddlewareBuilderLike, ResolverFunction};

// TODO: Deal with LayerCtx and context switching
pub trait ProcedureLike<TLayerCtx: Send + Sync + 'static> {
    type Middleware: AlphaMiddlewareBuilderLike<LayerContext = TLayerCtx> + Send;

    fn query<R2, R2Marker>(
        self,
        builder: R2,
    ) -> crate::alpha::procedure::AlphaProcedure<R2, R2Marker, Self::Middleware>
    where
        R2: ResolverFunction<R2Marker, LayerCtx = TLayerCtx> + Fn(TLayerCtx, R2::Arg) -> R2::Result;

    // TODO: `.with()`

    // // TODO: Use the `impl_procedure_like!()` if I can fix the visibility issue

    // // TODO: Using `self`

    // fn query<R, RMarker>(
    //     &self,
    //     builder: R,
    // ) -> AlphaProcedure<TCtx, Self::LayerCtx, R, RMarker, (), Self::Middleware>
    // where
    //     R: ResolverFunction<Self::LayerCtx, RMarker> + Fn(Self::LayerCtx, R::Arg) -> R::Result;
    // {
    //     AlphaProcedure::new_from_resolver(ProcedureKind::Query, builder)
    // }

    // fn mutation<R, RMarker>(
    //     &self,
    //     builder: R,
    // ) -> AlphaProcedure<TCtx, TCtx, R, RMarker, (), Self::Middleware>
    // where
    //     R: ResolverFunction<TCtx, RMarker> + Fn(TCtx, R::Arg) -> R::Result;
    // {
    //     AlphaProcedure::new_from_resolver(ProcedureKind::Mutation, builder)
    // }

    // TODO: `.subscription`
}

// This can be used on a type to allow it to be used without the `ProcedureLike` trait in scope.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_procedure_like {
    () => {
        pub fn query<R, RMarker>(
            self,
            builder: R,
        ) -> crate::alpha::procedure::AlphaProcedure<
            R,
            RMarker,
            crate::alpha::AlphaBaseMiddleware<TCtx>,
        >
        where
            R: ResolverFunction<RMarker, LayerCtx = TCtx> + Fn(TCtx, R::Arg) -> R::Result,
        {
            crate::alpha::procedure::AlphaProcedure::new_from_resolver(
                ProcedureKind::Query,
                self.1,
                builder,
            )
        }

        pub fn mutation<R, RMarker>(
            self,
            builder: R,
        ) -> crate::alpha::procedure::AlphaProcedure<
            R,
            RMarker,
            crate::alpha::AlphaBaseMiddleware<TCtx>,
        >
        where
            R: ResolverFunction<RMarker, LayerCtx = TCtx> + Fn(TCtx, R::Arg) -> R::Result,
        {
            crate::alpha::procedure::AlphaProcedure::new_from_resolver(
                ProcedureKind::Mutation,
                self.1,
                builder,
            )
        }

        // TODO: `.subscription`
    };
}

pub use crate::impl_procedure_like;
