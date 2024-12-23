use std::marker::PhantomData;

use rspc_core::State;

use crate::ProcedureMeta;

// TODO: `TError`?
// TODO: Explain executor order and why to use over `Middleware`?
pub struct Extension<TCtx, TInput, TResult> {
    pub(crate) setup: Option<Box<dyn FnOnce(&mut State, ProcedureMeta) + 'static>>,
    pub(crate) phantom: PhantomData<fn() -> (TCtx, TInput, TResult)>,
    // pub(crate) inner: Box<
    //     dyn FnOnce(
    //         MiddlewareHandler<TError, TNextCtx, TNextInput, TNextResult>,
    //     ) -> MiddlewareHandler<TError, TThisCtx, TThisInput, TThisResult>,
    // >,
}

// TODO: Debug impl

impl<TCtx, TInput, TResult> Extension<TCtx, TInput, TResult> {
    // TODO: Take in map function
    pub fn new() -> Self {
        Self {
            setup: None,
            phantom: PhantomData,
        }
    }

    // TODO: Allow multiple or error if defined multiple times?
    pub fn setup(mut self, func: impl FnOnce(&mut State, ProcedureMeta) + 'static) -> Self {
        self.setup = Some(Box::new(func));
        self
    }
}
