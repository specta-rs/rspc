use crate::middleware::MiddlewareHandler;

use super::{Procedure, ProcedureMeta};

// TODO: `pub(crate)` or `pub(super)`
// TODO: Rename this cause it's not really middleware related
// TODO: Maybe this should just be a type alias
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
        dyn FnOnce(
            ProcedureMeta,
            MiddlewareHandler<TError, TNextCtx, TNextInput, TNextResult>,
        ) -> Procedure<TCtx, TError>,
    >,
}
