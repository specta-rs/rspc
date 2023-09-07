use std::task::{Context, Poll};

use crate::RequestContext;

// TODO: Optimize the living hell outta this
// - Can we decode the entire request and then pass `&[u8]` to the function for the current batch item?
// - Can we pass down a buffer for the response that can be re-used between poll yields.
// - `size_hint` method

/// TODO
pub trait Executable<TLCtx>: 'static {
    // fn call(&self, req: RequestContext, etx: TLCtx, value: Vec<u8>);

    fn poll_chunk(&self, cx: Context<'_>) -> Poll<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assert the trait is dyn-safe
    #[allow(unused)]
    fn assert_is_dyn_safe<T: Executable<()> + Sized>(t: T) -> Box<dyn Executable<()>> {
        Box::new(t)
    }
}
