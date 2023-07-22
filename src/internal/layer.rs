use std::{convert::Infallible, future::ready, pin::Pin};

use bytes::Bytes;
use futures::{stream::once, Stream};
use http_body::Body;
use serde_json::Value;

use crate::{internal::middleware::RequestContext, ExecError};

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?
pub(crate) type FutureValueOrStream<'a> =
    Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'a>>;

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: SealedLayer<TLayerCtx> {}

pub(crate) type DynBody = dyn Body<Data = Bytes, Error = Infallible> + Unpin + Send + Sync;

mod private {
    use super::*;

    pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
        fn dyn_call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
            body: &mut DynBody,
        ) -> FutureValueOrStream<'_>;
    }

    impl<TLayerCtx: Send + 'static, L: Layer<TLayerCtx>> DynLayer<TLayerCtx> for L {
        fn dyn_call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
            body: &mut DynBody,
        ) -> FutureValueOrStream<'_> {
            match self.call(ctx, input, req, body) {
                Ok(stream) => Box::pin(stream),
                Err(err) => Box::pin(once(ready(Err(err)))),
            }
        }
    }

    /// Prevents the end user implementing the `Layer` trait and hides the internals
    pub trait SealedLayer<TLayerCtx: 'static>: DynLayer<TLayerCtx> {
        type Stream<'a>: Stream<Item = Result<Value, ExecError>> + Send + 'a;
        // type Body: Body<Data = Bytes, Error = Infallible> + Send + Sync + 'static; // TODO: ? -> Can middleware be the content type, technically but DX would be bad?

        fn call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
            body: &mut DynBody,
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
