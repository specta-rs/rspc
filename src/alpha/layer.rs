use std::{future::ready, pin::Pin};

use futures::{stream::once, Stream};
use serde_json::Value;

use crate::{internal::RequestContext, ExecError};

pub trait AlphaLayer<TLayerCtx: 'static>: DynLayer<TLayerCtx> + Send + Sync + 'static {
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

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?
pub type FutureValueOrStream<'a> =
    Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'a>>;

pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn dyn_call<'a>(&'a self, a: TLayerCtx, b: Value, c: RequestContext)
        -> FutureValueOrStream<'a>;
}

impl<TLayerCtx: Send + 'static, L: AlphaLayer<TLayerCtx>> DynLayer<TLayerCtx> for L {
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
