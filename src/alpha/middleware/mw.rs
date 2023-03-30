use std::{future::Future, marker::PhantomData};

use crate::alpha::MiddlewareArgMapper;

use super::{AlphaMiddlewareContext, MwV2Result};

pub trait MwV2<TLCtx, TMarker: Send>: Send + 'static {
    type Fut: Future<Output = Self::Result> + Send + 'static;
    type Result: MwV2Result<Ctx = Self::NewCtx>;
    type NewCtx: Send + Sync + 'static;

    // TODO: Rename
    fn run_me(
        &self,
        ctx: TLCtx,
        mw: AlphaMiddlewareContext<
            <<Self::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
        >,
    ) -> Self::Fut;
}

// TODO: This shouldn't be called a marker cause it's runtime
pub struct MwV2Marker<A, B>(PhantomData<(A, B)>);
impl<TLCtx, F, Fu, R> MwV2<TLCtx, MwV2Marker<Fu, R>> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(AlphaMiddlewareContext<<R::MwMapper as MiddlewareArgMapper>::State>, TLCtx) -> Fu
        + Send
        + 'static,
    Fu: Future<Output = R> + Send + 'static,
    R: MwV2Result<Ctx = TLCtx> + Send + 'static,
{
    type Fut = Fu;
    type Result = R;
    type NewCtx = TLCtx; // TODO: Make this work with context switching

    fn run_me(
        &self,
        ctx: TLCtx,
        mw: AlphaMiddlewareContext<<R::MwMapper as MiddlewareArgMapper>::State>,
    ) -> Self::Fut {
        self(mw, ctx)
    }
}
