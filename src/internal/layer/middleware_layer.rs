use std::{
    any::type_name,
    future::{ready, Future},
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{
    future::{Either, Flatten, IntoStream, Map, Ready},
    stream::{once, Once},
    FutureExt, Stream, StreamExt,
};
use serde_json::Value;

use crate::{
    error::ExecError,
    internal::middleware::{
        new_mw_ctx, IntoMiddlewareResult, MiddlewareFn, RequestContext,
        TODOTemporaryOnlyValidMarker,
    },
};

use super::Layer;

#[doc(hidden)]
pub struct MiddlewareLayer<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware> {
    // TODO: This `Arc` saves us a hole load of pain.
    pub(crate) next: Arc<TNextMiddleware>,
    pub(crate) mw: TNewMiddleware,
    pub(crate) phantom: PhantomData<(TLayerCtx, TNewCtx)>,
}

// type CallbackFn<TNewMiddleware, TLayerCtx, TNewCtx> = fn(
//     <TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result,
// ) -> Either<
//     <<TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result as IntoMiddlewareResult<
//         TODOTemporaryOnlyValidMarker,
//     >>::Stream,
//     Once<std::future::Ready<Result<Value, ExecError>>>,
// >;

impl<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware> Layer<TLayerCtx>
    for MiddlewareLayer<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TNewCtx: Send + Sync + 'static,
    TNextMiddleware: Layer<TNewCtx> + Sync + 'static,
    TNewMiddleware: MiddlewareFn<TLayerCtx, TNewCtx> + Send + Sync + 'static,
{
    // TODO: Lol Rustfmt can't handle this
    type Stream = MiddlewareLayerStream<futures::stream::Flatten<futures::future::IntoStream<futures::future::Map<<TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Future, 
    fn(
        <TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result,
    ) -> Either<   <<TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result as IntoMiddlewareResult<TODOTemporaryOnlyValidMarker>  >::Stream     , Once<std::future::Ready<Result<Value, ExecError>>>>

                        >>>>;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream, ExecError> {
        let callback: fn(
            <TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result,
        ) -> Either<   <<TNewMiddleware as MiddlewareFn<TLayerCtx, TNewCtx>>::Result as IntoMiddlewareResult<TODOTemporaryOnlyValidMarker>  >::Stream     , Once<std::future::Ready<Result<Value, ExecError>>>> =
            |f| match f.into_result() {
                Ok(result) =>  Either::Left(result),
                Err(err) => Either::Right(once(ready(Err(err)))),
            };

        let mw = self
            .mw
            .execute(ctx, new_mw_ctx(input, req))
            .map(callback)
            .into_stream()
            .flatten();

        Ok(MiddlewareLayerStream {
            mw,
            phantom: PhantomData,
        })
    }
}

pub struct MiddlewareLayerStream<S> {
    mw: S,
    phantom: PhantomData<S>,
}

impl<S> Stream for MiddlewareLayerStream<S>
where
    S: Stream<Item = Result<Value, ExecError>>,
{
    type Item = Result<Value, ExecError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}
