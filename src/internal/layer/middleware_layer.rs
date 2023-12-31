use std::{
    future::{ready, Ready},
    marker::PhantomData,
    sync::Arc,
};

use futures::{
    future::Either,
    stream::{once, Once},
    FutureExt, StreamExt,
};
use serde_json::Value;

use crate::{
    error::ExecError,
    internal::middleware::{
        new_mw_ctx, IntoMiddlewareResult, MiddlewareFn, RequestContext,
        TODOTemporaryOnlyValidMarker,
    },
};

use super::{middleware_layer_stream::MiddlewareInterceptor, Layer};

#[doc(hidden)]
pub struct MiddlewareLayer<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware> {
    // TODO: This `Arc` saves us a hole load of pain.
    pub(crate) next: Arc<TNextMiddleware>,
    pub(crate) mw: TNewMiddleware,
    pub(crate) phantom: PhantomData<(TLayerCtx, TNewCtx)>,
}

type CallbackFn<TNewMiddleware, TLayerCtx, TNewCtx> = fn(
    <TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result,
) -> Either<
    <<TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result as IntoMiddlewareResult<
        TODOTemporaryOnlyValidMarker,
    >>::Stream,
    Once<Ready<Result<Value, ExecError>>>,
>;

impl<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware> Layer<TLayerCtx>
    for MiddlewareLayer<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TNewCtx: Send + Sync + 'static,
    TNextMiddleware: Layer<TNewCtx> + Sync + 'static,
    TNewMiddleware: MiddlewareFn<TLayerCtx, TNewCtx> + Send + Sync + 'static,
{
    type Stream = MiddlewareInterceptor<
        futures::stream::Flatten<
            futures::future::IntoStream<
                futures::future::Map<
                    <TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Future,
                    CallbackFn<TNewMiddleware, TLayerCtx, TNewCtx>,
                >,
            >,
        >,
        TNextMiddleware,
        TNewCtx,
    >;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream, ExecError> {
        let callback: CallbackFn<TNewMiddleware, TLayerCtx, TNewCtx> = |f| match f.into_result() {
            Ok(result) => Either::Left(result),
            Err(err) => Either::Right(once(ready(Err(err)))),
        };

        let mw = self
            .mw
            .execute(ctx, new_mw_ctx(input, req))
            .map(callback)
            .into_stream()
            .flatten();

        Ok(MiddlewareInterceptor {
            mw,
            next: self.next.clone(),
            stream: None,
            phantom: PhantomData,
        })
    }
}
