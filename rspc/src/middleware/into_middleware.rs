use std::marker::PhantomData;

use crate::{Error, Extension, ProcedureBuilder};

use super::Middleware;

// TODO: Expose in public API or seal???
// TODO: This API could lead to bad errors
pub trait IntoMiddleware<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult> {
    type TNextCtx;
    type I;
    type R;

    fn build(
        self,
        this: ProcedureBuilder<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult>,
    ) -> ProcedureBuilder<TError, TRootCtx, Self::TNextCtx, TBaseInput, Self::I, TBaseResult, Self::R>;
}

impl<TError, TRootCtx, TCtx, TInput, TResult, TNextCtx, TBaseInput, I, TBaseResult, R>
    IntoMiddleware<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult>
    for Middleware<TError, TCtx, TInput, TResult, TNextCtx, I, R>
where
    // TODO: This stuff could lead to bad errors
    // TODO: Could we move them onto the function instead and constrain on `with` too???
    TError: Error,
    TRootCtx: 'static,
    TNextCtx: 'static,
    TCtx: 'static,
    TInput: 'static,
    TResult: 'static,
    TBaseInput: 'static,
    I: 'static,
    TBaseResult: 'static,
    R: 'static,
{
    type TNextCtx = TNextCtx;
    type I = I;
    type R = R;

    fn build(
        self,
        this: ProcedureBuilder<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult>,
    ) -> ProcedureBuilder<TError, TRootCtx, Self::TNextCtx, TBaseInput, Self::I, TBaseResult, Self::R>
    {
        ProcedureBuilder {
            build: Box::new(|ty, mut setups, handler| {
                if let Some(setup) = self.setup {
                    setups.push(setup);
                }

                (this.build)(ty, setups, (self.inner)(handler))
            }),
            phantom: PhantomData,
        }
    }
}

// TODO: Constrain to base types
impl<TError, TRootCtx, TCtx, TBaseInput, I, TBaseResult, R>
    IntoMiddleware<TError, TRootCtx, TCtx, TBaseInput, I, TBaseResult, R>
    for Extension<TCtx, TBaseInput, TBaseResult>
where
    // TODO: This stuff could lead to bad errors
    // TODO: Could we move them onto the function instead and constrain on `with` too???
    TError: Error,
    TRootCtx: 'static,
    TCtx: 'static,
    TBaseInput: 'static,
    I: 'static,
    TBaseResult: 'static,
    R: 'static,
{
    type TNextCtx = TCtx;
    type I = I;
    type R = R;

    fn build(
        self,
        this: ProcedureBuilder<TError, TRootCtx, TCtx, TBaseInput, I, TBaseResult, R>,
    ) -> ProcedureBuilder<TError, TRootCtx, Self::TNextCtx, TBaseInput, Self::I, TBaseResult, Self::R>
    {
        ProcedureBuilder {
            build: Box::new(|ty, mut setups, handler| {
                if let Some(setup) = self.setup {
                    setups.push(setup);
                }

                (this.build)(ty, setups, handler)
            }),
            phantom: PhantomData,
        }
    }
}
