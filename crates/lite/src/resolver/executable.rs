use std::{
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::Executable;

// TODO: Private or sealed
pub(crate) struct Resolver<F, A, Fu> {
    phantom: PhantomData<(F, A, Fu)>,
}

impl<TLCtx, F, A, Fu> Executable<TLCtx> for Resolver<F, A, Fu>
where
    F: Fn(TLCtx, A) -> Fu + 'static,
    A: DeserializeOwned + Type + 'static,
    Fu: Future + 'static,
{
    // fn call(&self, ctx: TLCtx, value: Vec<u8>) {
    //     todo!()
    // }

    fn poll_chunk(&self, cx: Context<'_>) -> Poll<Vec<u8>> {
        // let _span = this.span.as_ref().map(|s| s.enter()); // TODO: Global hook for tracing-style stuff without needing direct integration

        // match serde_json::from_slice(&value) {
        //     Ok(value) => self.call(ctx, value),
        //     Err(err) => {}
        // }

        todo!();
    }
}
