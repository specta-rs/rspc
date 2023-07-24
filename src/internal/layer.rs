use std::{future::ready, pin::Pin};

use serde_json::Value;

use super::exec::RspcStream;
use crate::{
    internal::{exec::Once, middleware::RequestContext},
    ExecError,
};

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?
pub(crate) type FutureValueOrStream<'a> =
    Pin<Box<dyn RspcStream<Item = Result<Value, ExecError>> + Send + 'a>>;

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: SealedLayer<TLayerCtx> {}

mod private {
    use super::*;

    pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
        fn dyn_call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
        ) -> FutureValueOrStream<'_>;
    }

    impl<TLayerCtx: Send + 'static, L: Layer<TLayerCtx>> DynLayer<TLayerCtx> for L {
        fn dyn_call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
        ) -> FutureValueOrStream<'_> {
            match self.call(ctx, input, req) {
                Ok(stream) => Box::pin(stream),
                // TODO: Avoid allocating error future here
                Err(err) => Box::pin(Once::new(ready(Err(err)))),
            }
        }
    }

    /// Prevents the end user implementing the `Layer` trait and hides the internals
    pub trait SealedLayer<TLayerCtx: 'static>: DynLayer<TLayerCtx> {
        type Stream<'a>: RspcStream<Item = Result<Value, ExecError>> + Send + 'a;

        fn call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
        ) -> Result<Self::Stream<'_>, ExecError>;

        fn erase(self) -> Box<dyn DynLayer<TLayerCtx>>
        where
            Self: Sized,
        {
            Box::new(self)
        }
    }

    impl<TLayerCtx: 'static, L: SealedLayer<TLayerCtx>> Layer<TLayerCtx> for L {}
}

pub(crate) use private::{DynLayer, SealedLayer};
