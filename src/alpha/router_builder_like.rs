use std::borrow::Cow;

use super::IntoProcedure;

pub(crate) type ProcedureList<TCtx> = Vec<(Cow<'static, str>, Box<dyn IntoProcedure<TCtx>>)>;

pub trait AlphaRouterBuilderLike<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    fn procedures(self) -> ProcedureList<TCtx>;
}

// TODO: Add legacy interop?
// impl<TCtx, TMeta, TMiddleware> AlphaRouterBuilderLike<TCtx>
//     for crate::RouterBuilder<TCtx, TMeta, TMiddleware>
// where
//     TCtx: Send + Sync + 'static,
//     TMeta: Send + 'static,
//     TMiddleware: MiddlewareBuilderLike<TCtx> + Send + 'static,
// {
// }
