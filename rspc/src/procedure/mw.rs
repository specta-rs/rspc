use crate::middleware::MiddlewareInner;

use super::{exec_input::InputValueInner, InternalError, ProcedureStream};

pub(super) type InvokeFn<TCtx, TErr> =
    Box<dyn Fn(TCtx, &mut dyn InputValueInner) -> Result<ProcedureStream<TErr>, InternalError>>;

pub(super) struct Mw<
    // Must be same through whole chain
    TError: std::error::Error,
    TCtx,
    // From the current layer we are storing
    TNextCtx,
    TNextInput,
    TNextResult,
> {
    // TODO: I think it would be more logical for the argument to be just `MiddlewareInner.handler`. Parsing `setup` around adds no value and is plain confusing.
    pub build: Box<
        dyn FnOnce(MiddlewareInner<TNextCtx, TNextInput, TNextResult>) -> InvokeFn<TCtx, TError>,
    >,
}

// impl<TError, TCtx, TNextCtx, TNextInput, TNextResult>
//     Mw<TError, TCtx, TNextCtx, TNextInput, TNextResult>
// where
//     TError: std::error::Error,
//     TNextCtx: 'static,
//     TNextInput: 'static,
//     TNextResult: 'static,
// {
//     // pub fn new(
//     //     build: impl Fn(MiddlewareInner<TNextCtx, TNextInput, TNextResult>) -> InvokeFn<TCtx, TError>
//     //         + 'static,
//     // ) -> Self {
//     //     Self {
//     //         build: Box::new(build),
//     //     }
//     // }
// }
