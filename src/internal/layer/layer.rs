use futures::{FutureExt, Stream, StreamExt};
use serde_json::Value;
use std::{future::Future, pin::Pin};

use crate::{
    error::ExecError,
    internal::{layer::layer_result::DynLayerResult, middleware::RequestContext},
};

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> impl Future<
        Output = Result<impl Stream<Item = Result<Value, ExecError>> + Send + 'static, ExecError>,
    > + Send;
}

// TODO: Replace this with `rspc_core::Procedure` if possible
pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> DynLayerResult<'_, Result<Value, ExecError>>;
    // ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + '_>>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn dyn_call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
        // TODO: We gotta get rid of the lifetime
        // ) -> Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + '_>> {
    ) -> DynLayerResult<'_, Result<Value, ExecError>> {
        // Box::pin(
        //     async move {
        //         match self.call(ctx, input, req).await {
        //             Ok(stream) => stream,
        //             // TODO: Avoid allocating error future here
        //             Err(err) => todo!(), // Box::pin(once(ready(Err(err)))),
        //         }
        //     }
        //     .into_stream()
        //     .flatten(),
        // )

        DynLayerResult::new(async move {
            match self.call(ctx, input, req).await {
                Ok(stream) => stream,
                // TODO: Avoid allocating error future here
                Err(err) => todo!(), // Box::pin(once(ready(Err(err)))),
            }
        })
    }
}
