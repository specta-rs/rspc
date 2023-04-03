use std::future::Future;

use serde::{de::DeserializeOwned, Serialize};
use specta::Type;

use super::{AlphaMiddlewareContext, MwV2, MwV2Result};

/// TODO
pub trait MiddlewareArgMapper: Send + Sync {
    /// TODO
    type State: Send + Sync + 'static;

    /// TODO
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    /// TODO
    type Output<T>: Serialize
    where
        T: Serialize;

    /// TODO
    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State);
}

/// TODO
pub enum MiddlewareArgMapperPassthrough {}

impl MiddlewareArgMapper for MiddlewareArgMapperPassthrough {
    type State = ();
    type Input<T> = T
    where
        T: DeserializeOwned + Type + 'static;
    type Output<T> = T where T: Serialize;

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State) {
        (arg, ())
    }
}

// TODO: This is fairly cringe but will be improved.
// TODO: Split `TMwMapper` and other generic so this is safe for userspace
pub fn arg_mapper_mw<TMwMapper, TLCtx, Fu>(
    handler: impl Fn(AlphaMiddlewareContext, TLCtx, TMwMapper::State) -> Fu + Send + Sync + 'static,
) -> impl MwV2<TLCtx, NewCtx = <Fu::Output as MwV2Result>::Ctx>
where
    TMwMapper: MiddlewareArgMapper,
    TLCtx: Send + Sync + 'static,
    Fu: Future + Send + Sync + 'static,
    Fu::Output: MwV2Result + Send + 'static,
{
    // TODO: Make this passthrough to new handler but provide the owned `State` as an arg
    move |mw, ctx| {
        let (out, state) =
            TMwMapper::map::<serde_json::Value>(serde_json::from_value(mw.input).unwrap());

        handler(
            AlphaMiddlewareContext {
                input: serde_json::to_value(out).unwrap(), // TODO: Error handling
                req: mw.req,
                _priv: (),
            },
            ctx,
            state,
        )
    }
}
