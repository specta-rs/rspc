use std::{marker::PhantomData, ops::Deref};

/// TODO
pub struct Context<TCtx> {
    pub(crate) ctx: TCtx,
}

// This is a Rust anti-pattern but it allows extensions to generically extend the context so imo it's fine.
impl<TCtx> Deref for Context<TCtx> {
    type Target = TCtx;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

/// TODO
pub struct SubscriptionContext<TCtx, TResult> {
    pub(crate) ctx: TCtx,
    pub(crate) phantom: PhantomData<TResult>,
}

// This is a Rust anti-pattern but it allows extensions to generically extend the context so imo it's fine.
impl<TCtx, TResult> Deref for SubscriptionContext<TCtx, TResult> {
    type Target = TCtx;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<TCtx, TResult> SubscriptionContext<TCtx, TResult> {
    pub async fn next(&self) -> Option<()> {
        unimplemented!();
    }

    pub async fn emit(&self, _data: TResult) {
        unimplemented!();
    }
}
