use std::{fmt, marker::PhantomData};

/// a reference to the next thing to be executed in the procedure.
///
/// As we always execute top to bottom this will be the next middleware or the final resolver function.
///
/// Generics:
///  - `R` - // TODO
///  - `I` - // TODO
///  - `TCtx` - // TODO
///
pub struct Next<R, I, TCtx>(PhantomData<(R, I, TCtx)>);

impl<R, I, TCtx> fmt::Debug for Next<R, I, TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Next").finish()
    }
}

impl<R, I, TCtx> Next<R, I, TCtx> {
    /// execute the next thing in the chain and return the result
    pub async fn exec(self, ctx: TCtx, input: I) -> R {
        todo!();
    }
}
