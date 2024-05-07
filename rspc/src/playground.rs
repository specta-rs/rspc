//! TODO: Remove this file
//!

use std::{future::Future, marker::PhantomData};

use crate::procedure::Next;

// TODO: Make the `Next = ` default using GATs to have default or override

// TODO: Seal?
// TODO: Rename?

pub trait NextTrait {
    type Ctx;
    type Input;
    type Output;
}

impl<TCtx, I, R> NextTrait for Next<TCtx, I, R> {
    type Ctx = TCtx;
    type Input = I;
    type Output = R;
}

pub trait Middleware<Next: NextTrait> {
    type Ctx;
    type Input;
}

struct StandaloneMiddleware<N, Ctx, I> {
    phantom: PhantomData<(N, Ctx, I)>,
}

impl<N: NextTrait, Ctx, I> Middleware<N> for StandaloneMiddleware<N, Ctx, I> {
    type Ctx = Ctx;
    type Input = I;
}

// TODO: Don't hardcode next result to `i32`
pub fn mw<TCtx, I, R, F, NextCtx, NextI>(
    _: impl Fn(TCtx, I, Next<i32, NextCtx, NextI>) -> F,
) -> impl Middleware<Next<i32, NextCtx, NextI>>
where
    F: Future<Output = R>,
{
    StandaloneMiddleware::<_, NextCtx, NextI> {
        phantom: PhantomData,
    }
}
