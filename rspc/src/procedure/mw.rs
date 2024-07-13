use crate::middleware::MiddlewareHandler;

use super::procedure::InvokeFn;

// TODO: `pub(crate)` or `pub(super)`
pub(crate) struct Mw<
    // Must be same through whole chain
    TError,
    TCtx,
    // From the current layer we are storing
    TNextCtx,
    TNextInput,
    TNextResult,
> {
    pub build: Box<
        dyn FnOnce(MiddlewareHandler<TNextCtx, TNextInput, TNextResult>) -> InvokeFn<TCtx, TError>,
    >,
}
