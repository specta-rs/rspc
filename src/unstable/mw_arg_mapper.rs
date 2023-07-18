use std::{future::Future, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use specta::Type;

// TODO: This should be possible without `internal` API's
use crate::internal::{
    middleware::Middleware,
    middleware::{MiddlewareContext, MwV2Result},
};

/// A trait for modifying a procedures argument type.
///
/// This trait primarily exists to workaround Rust's lack of generic closures.
///
/// To explain it more say you had `{ library_id: Uuid, data: T }` as your input from the frontend.
/// Your `Self::State` would be `Uuid` and your `Self::Output<T>` would be `T`.
/// This way `Self::State` can be passed into the middleware closure "erasing" the generic `T`.
///
/// This is very powerful for multi-tenant applications but also breaks all rspc typesafe guarantees.
pub trait MwArgMapper: Send + Sync {
    /// the output of the mapper for consumption in your middleware.
    type State: Send + Sync + 'static;

    /// the output of the mapper to be passed on to the following procedure.
    ///
    /// WARNING: This is not typesafe. If you get it wrong it will runtime panic!
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    /// Apply the mapping to the input argument.
    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (T, Self::State);
}

/// A middleware that allows you to modify the input arguments of a procedure.
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
        TNCtx: Send + Sync + 'static,
        Fu: Future<Output = R> + Send + Sync + 'static,
        R: MwV2Result<Ctx = TNCtx> + Send + 'static,
    {
        // TODO: Make this passthrough to new handler but provide the owned `State` as an arg
        private::MiddlewareFnWithTypeMapper(
            move |mw: MiddlewareContext, ctx| {
                let (out, state) =
                    M::map::<serde_json::Value>(serde_json::from_value(mw.input).unwrap()); // TODO: Error handling

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
