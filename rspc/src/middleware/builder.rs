use std::{any::Any, future::Future, marker::PhantomData};

use crate::procedure::Next;

// TODO: Access state in `start`
// TODO: Access state in `with`

/// TODO
pub struct MiddlewareBuilder<TCtx, I, N, R> {
    phantom: PhantomData<(TCtx, I, N, R)>,
}

impl<TCtx, I, NextR, NextI, TNextCtx, R>
    MiddlewareBuilder<TCtx, I, Next<NextR, NextI, TNextCtx>, R>
{
    /// TODO
    pub fn builder() -> Self {
        todo!();
    }

    /// TODO
    pub fn start(self, func: impl FnOnce()) -> Self {
        // TODO

        self
    }

    /// TODO
    pub fn state<T: Any>(self, state: T) -> Self {
        // TODO

        self
    }

    /// TODO
    pub fn with<F>(&self, handler: impl Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> F) -> Self
    where
        F: Future<Output = R>,
    {
        todo!();
    }
}
