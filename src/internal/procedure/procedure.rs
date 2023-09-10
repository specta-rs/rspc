use std::marker::PhantomData;

use crate::internal::middleware::{
    ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder,
};

/// TODO: Explain
pub struct MissingResolver<TError>(PhantomData<TError>);

impl<TError> Default for MissingResolver<TError> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// TODO
pub struct Procedure<T, TMiddleware> {
    pub(crate) resolver: T,
    pub(crate) mw: TMiddleware,
}

impl<T, TMiddleware> Procedure<T, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn new(resolver: T, mw: TMiddleware) -> Self {
        Self { resolver, mw }
    }
}

macro_rules! resolver {
    ($func:ident, $kind:ident) => {
        // TODO: Bring this back
        // pub fn $func<R, M>(self, resolver: R) -> Procedure<HasResolver<R, TResult, M>, TMiddleware>
        // where
        //     HasResolver<R, TResultM>: ResolverFunction<TMiddleware::LayerCtx, TError>,
        // {
        //     Procedure::new(HasResolver::new(resolver, ProcedureKind::$kind), self.mw)
        // }
    };
}

// Can only set the resolver or add middleware until a resolver has been set.
// Eg. `.query().subscription()` makes no sense.
impl<TMiddleware, TError> Procedure<MissingResolver<TError>, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    resolver!(query, Query);
    resolver!(mutation, Mutation);
    resolver!(subscription, Subscription);

    pub fn error(self) -> Procedure<MissingResolver<TError>, TMiddleware> {
        Procedure {
            resolver: self.resolver,
            mw: self.mw,
        }
    }

    pub fn with<Mw: ConstrainedMiddleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError>, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: crate::internal::middleware::Middleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError>, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }
}
