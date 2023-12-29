use std::marker::PhantomData;

use crate::{
    error::{private::IntoResolverError, Infallible},
    internal::{
        layer::{BaseLayer, MiddlewareLayerBuilder},
        middleware::Middleware,
    },
    procedure::{MissingResolver, Procedure},
    router_builder::RouterBuilder,
};

/// Rspc is a starting point for constructing rspc procedures or routers.
///
/// This method supports const contexts so it can be instantiated at the top level as reuse across the whole application.
///
/// ```rust
/// use rspc::Rspc;
///
/// const R: Rspc<()> = Rspc::new();
/// ```
pub struct Rspc<
    TCtx = (), // The is the context the current router was initialised with,
    TError = Infallible,
> where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    phantom: PhantomData<(TCtx, TError)>,
}

#[allow(clippy::new_without_default)]
impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    pub fn router(&self) -> RouterBuilder<TCtx> {
        RouterBuilder::_internal_new()
    }

    pub fn error<TNewError>(self) -> Procedure<MissingResolver<TNewError, BaseLayer<TCtx>>> {
        MissingResolver::new(BaseLayer::default())
    }

    pub fn with<Mw: Middleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError, MiddlewareLayerBuilder<BaseLayer<TCtx>, Mw>>> {
        MissingResolver::new(MiddlewareLayerBuilder {
            middleware: BaseLayer::default(),
            mw,
        })
    }

    // TODO:
    // resolvers!(_, TCtx, BaseMiddleware<TCtx>, BaseMiddleware::default());
}
