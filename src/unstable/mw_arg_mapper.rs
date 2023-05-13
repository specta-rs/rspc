use std::{future::Future, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use specta::Type;

// TODO: This should be possible without `internal` API's
use crate::internal::{middleware::Middleware, MiddlewareContext, MwV2Result};

/// TODO
pub trait MwArgMapper: Send + Sync {
    /// TODO
    type State: Send + Sync + 'static;

    /// TODO
    ///
    /// This is not typesafe. If you get it wrong it will runtime panic!
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    /// TODO
    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (T, Self::State);
}

pub struct MwArgMapperMiddleware<M: MwArgMapper>(PhantomData<M>);

impl<M: MwArgMapper + 'static> MwArgMapperMiddleware<M> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    pub fn mount<TLCtx, TNCtx, Fu, R>(
        &self,
        handler: impl Fn(MiddlewareContext, TLCtx, M::State) -> Fu + Send + Sync + 'static,
    ) -> impl Middleware<TLCtx, NewCtx = TNCtx>
    where
        TLCtx: Send + Sync + 'static,
        Fu: Future<Output = R> + Send + Sync + 'static,
        R: MwV2Result<Ctx = TNCtx> + Send + 'static,
    {
        // TODO: Make this passthrough to new handler but provide the owned `State` as an arg
        private::MiddlewareFnWithTypeMapper(
            move |mw: MiddlewareContext, ctx| {
                let (out, state) =
                    M::map::<serde_json::Value>(serde_json::from_value(mw.input).unwrap());

                handler(
                    MiddlewareContext {
                        input: serde_json::to_value(out).unwrap(), // TODO: Error handling
                        req: mw.req,
                        _priv: (),
                    },
                    ctx,
                    state,
                )
            },
            PhantomData::<M>,
        )
    }
}

mod private {
    use crate::internal::middleware::SealedMiddleware;

    use super::*;

    pub struct MiddlewareFnWithTypeMapper<M, F>(pub(super) F, pub(super) PhantomData<M>);

    impl<M, TLCtx, F, Fu, R> SealedMiddleware<TLCtx> for MiddlewareFnWithTypeMapper<M, F>
    where
        TLCtx: Send + Sync + 'static,
        F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
        Fu: Future<Output = R> + Send + 'static,
        R: MwV2Result + Send + 'static,
        M: MwArgMapper + 'static,
    {
        type Fut = Fu;
        type Result = R;
        type NewCtx = R::Ctx; // TODO: Make this work with context switching
        type Arg<T: Type + DeserializeOwned + 'static> = M::Input<T>;

        fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
            (self.0)(mw, ctx)
        }
    }
}
