//! This comment contains an overview of the rationale behind the design of the middleware system.
//! NOTE: It is *not* included in the generated Rust docs!
//!
//! For future reference:
//!
//! Having a standalone middleware that is like `fn my_middleware() -> impl Middleware<...>` results in *really* bad error messages.
//! This is because the middleware is defined within the function and then *constrained* at the function boundary.
//! These places are different so the compiler is like lol trait xyz with generics iop does match the trait xyz with generics abc.
//!
//! Instead if the builder function takes a [`MiddlewareBuilder`] the constrain it applied prior to the middleware being defined.
//! This allows the compiler to constrain the types at the middleware definition site which leads to insanely better error messages.
//!
//! Be aware this talk about constraining and definition is just me speaking about what I have observed.
//! This could be completely wrong from a technical perspective.
//!
//! TODO: Explaining why inference across boundaries is not supported.
//!
//! TODO: Explain why we can't have `fn mw(...) -> Middleware` -> It's because of default generics!!!
//!
//! TODO: Why we can't use `const`'s for declaring middleware -> Boxing

use std::{pin::Pin, sync::Arc};

use futures::Future;

use crate::rewrite::{procedure::ProcedureMeta, State};

use super::Next;

pub(crate) type MiddlewareHandler<TError, TNextCtx, TNextInput, TNextResult> = Box<
    dyn Fn(
            TNextCtx,
            TNextInput,
            ProcedureMeta,
        ) -> Pin<Box<dyn Future<Output = Result<TNextResult, TError>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// An abstraction for common logic that can be applied to procedures.
///
/// A middleware can be used to run custom logic and modify the context, input, and result of the next procedure. This makes is perfect for logging, authentication and many other things!
///
/// Middleware are applied with [ProcedureBuilder::with](crate::procedure::ProcedureBuilder::with).
///
/// # Generics
///
/// - `TError` - The type of the error that can be returned by the middleware. Defined by [ProcedureBuilder::error](crate::procedure::ProcedureBuilder::error).
/// - `TThisCtx` - // TODO
/// - `TThisInput` - // TODO
/// - `TThisResult` - // TODO
/// - `TNextCtx` - // TODO
/// - `TNextInput` - // TODO
/// - `TNextResult` - // TODO
///
/// TODO: [
//  Context of previous layer (`ctx`),
//  Error type,
//  The input to the middleware (`input`),
//  The result of the middleware (return type of future),
//  - This following will default to the input types if not explicitly provided // TODO: Will this be confusing or good?
//  The context returned by the middleware (`next.exec({dis_bit}, ...)`),
//  The input to the next layer (`next.exec(..., {dis_bit})`),
//  The result of the next layer (`let _result: {dis_bit} = next.exec(...)`),
// ]
///
/// ```rust
/// TODO: Example to show where the generics line up.
/// ```
///
/// # Stacking
///
/// TODO: Guide the user through stacking.
///
/// # Example
///
/// TODO:
///
// TODO: Explain why they are required -> inference not supported across boundaries.
pub struct Middleware<
    TError,
    TThisCtx,
    TThisInput,
    TThisResult,
    TNextCtx = TThisCtx,
    TNextInput = TThisInput,
    TNextResult = TThisResult,
> {
    pub(crate) setup: Option<Box<dyn FnOnce(&mut State, ProcedureMeta) + 'static>>,
    pub(crate) inner: Box<
        dyn FnOnce(
            MiddlewareHandler<TError, TNextCtx, TNextInput, TNextResult>,
        ) -> MiddlewareHandler<TError, TThisCtx, TThisInput, TThisResult>,
    >,
}

// TODO: Debug impl

impl<TError, TThisCtx, TThisInput, TThisResult, TNextCtx, TNextInput, TNextResult>
    Middleware<TError, TThisCtx, TThisInput, TThisResult, TNextCtx, TNextInput, TNextResult>
where
    TError: 'static,
    TNextCtx: 'static,
    TNextInput: 'static,
    TNextResult: 'static,
{
    pub fn new<F: Future<Output = Result<TThisResult, TError>> + Send + 'static>(
        func: impl Fn(TThisCtx, TThisInput, Next<TError, TNextCtx, TNextInput, TNextResult>) -> F
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            setup: None,
            inner: Box::new(move |next| {
                // TODO: Don't `Arc<Box<_>>`
                let next = Arc::new(next);

                Box::new(move |ctx, input, meta| {
                    let f = func(
                        ctx,
                        input,
                        Next {
                            meta,
                            next: next.clone(),
                        },
                    );

                    Box::pin(f)
                })
            }),
        }
    }

    pub fn setup(mut self, func: impl FnOnce(&mut State, ProcedureMeta) + 'static) -> Self {
        self.setup = Some(Box::new(func));
        self
    }
}
