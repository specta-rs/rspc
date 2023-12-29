use std::{
    future::{ready, IntoFuture},
    marker::PhantomData,
};

use futures::{future::Either, FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use serde_json::Value;

use crate::{
    error::ExecError,
    internal::middleware::{new_mw_ctx, Executable2, Middleware, MwV2Result, RequestContext},
};

use super::Layer;

#[doc(hidden)]
pub struct MiddlewareLayer<TLayerCtx, TNextLayer, TNewMiddleware> {
    pub(crate) next: TNextLayer,
    pub(crate) mw: TNewMiddleware,
    pub(crate) phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx, TNextMiddleware, TNewMiddleware> Layer<TLayerCtx>
    for MiddlewareLayer<TLayerCtx, TNextMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TNextMiddleware: Layer<TNewMiddleware::NewCtx> + Sync + 'static,
    TNewMiddleware: Middleware<TLayerCtx> + Send + Sync + 'static,
{
    async fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<impl Stream<Item = Result<Value, ExecError>> + Send, ExecError> {
        let (ctx, input, req, resp_fn) = self
            .mw
            .run_me(ctx, new_mw_ctx(input, req))
            .await
            .explode()?;

        self.next.call(ctx, input, req).await.map(move |stream| {
            stream.and_then(move |v| {
                let v = match &resp_fn {
                    Some(resp_fn) => Either::Left(resp_fn.call(v)),
                    None => Either::Right(v),
                };

                async move {
                    match v {
                        Either::Left(v) => Ok(v.await),
                        Either::Right(v) => Ok(v),
                    }
                }
            })
        })
    }
}
