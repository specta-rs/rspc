use std::{future::ready, pin::Pin};

use futures::{stream::once, Stream};
use serde_json::Value;

use crate::{internal::RequestContext, ExecError};

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?
pub(crate) type FutureValueOrStream<'a> =
    Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'a>>;

/// TODO
pub trait Layer<TLayerCtx: 'static>: private::SealedLayer<TLayerCtx> {}

mod private {
    use super::*;

    pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
        fn dyn_call<'a>(
            &'a self,
            a: TLayerCtx,
            b: Value,
            c: RequestContext,
        ) -> FutureValueOrStream<'a>;
    }

    impl<TLayerCtx: Send + 'static, L: Layer<TLayerCtx>> DynLayer<TLayerCtx> for L {
        fn dyn_call<'a>(
            &'a self,
            a: TLayerCtx,
            b: Value,
            c: RequestContext,
        ) -> FutureValueOrStream<'a> {
            match self.call(a, b, c) {
                Ok(stream) => Box::pin(stream),
                Err(err) => Box::pin(once(ready(Err(err)))),
            }
        }
    }

    /// Prevents the end user implementing the `Layer` trait and hides some of the internals
    pub trait SealedLayer<TLayerCtx: 'static>: DynLayer<TLayerCtx> {
        type Stream<'a>: Stream<Item = Result<Value, ExecError>> + Send + 'a;

        fn call<'a>(
            &'a self,
            a: TLayerCtx,
            b: Value,
            c: RequestContext,
        ) -> Result<Self::Stream<'a>, ExecError>;

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
