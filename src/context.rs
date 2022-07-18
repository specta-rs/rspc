use std::ops::Deref;

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
