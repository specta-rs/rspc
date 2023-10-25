mod private {
    use std::{
        borrow::Cow,
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    use futures::{future, pin_mut, stream, Future, FutureExt, Stream, StreamExt};
    use pin_project_lite::pin_project;
    use serde_json::Value;
    use specta::{ts, TypeMap};

    use crate::internal::middleware::{Middleware, SealedMiddleware};
    use rspc_core::{
        error::ExecError,
        internal::{
            new_mw_ctx, Body, Executable2, ExplodedMwResult, Layer, MwV2Result, PinnedOption,
            PinnedOptionProj, ProcedureDef, RequestContext,
        },
    };

    #[doc(hidden)]
    pub struct MiddlewareLayer<TLayerCtx, TNextLayer, TNewMiddleware> {
        pub(crate) next: TNextLayer,
        pub(crate) mw: TNewMiddleware,
        pub(crate) phantom: PhantomData<TLayerCtx>,
    }

    impl<TLayerCtx, TNextMiddleware, TNewMiddleware> Layer<TLayerCtx>
        for MiddlewareLayer<TLayerCtx, TNextMiddleware, TNewMiddleware>
    where
        TLayerCtx: SendSyncStatic,
        TNextMiddleware: Layer<TNewMiddleware::NewCtx> + Sync + 'static,
        TNewMiddleware: Middleware<TLayerCtx> + Send + Sync + 'static,
    {
        type Stream<'a> = MiddlewareLayerFuture<'a, TLayerCtx, TNewMiddleware, TNextMiddleware>;

        fn into_procedure_def(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, ts::ExportError> {
            self.next.into_procedure_def(key, ty_store)
        }

        fn call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
        ) -> Result<Self::Stream<'_>, ExecError> {
            let fut = self.mw.run_me(ctx, new_mw_ctx(input, req));

            Ok(MiddlewareLayerFuture {
                inner: middleware_layer_future::<TLayerCtx, TNewMiddleware, TNextMiddleware>(
                    fut, &self.next,
                ),
            })
        }
    }

    // This exists because `pin_project_lite` doesn't understand `+` bounds
    pub trait SendSyncStatic: Send + Sync + 'static {}
    impl<T: Send + Sync + 'static> SendSyncStatic for T {}

    type LayerStream<'a, TLayerCtx, TMiddleware, TNextLayer> =
        <TNextLayer as Layer<<TMiddleware as SealedMiddleware<TLayerCtx>>::NewCtx>>::Stream<'a>;
    type LayerItem<'a, TLayerCtx, TMiddleware, TNextLayer> =
        <LayerStream<'a, TLayerCtx, TMiddleware, TNextLayer> as Stream>::Item;

    type Bruh<'a, TLayerCtx, TMiddleware, TNextLayer> = future::Either<
        future::Map<
            RespFut<TLayerCtx, TMiddleware>,
            fn(Value) -> LayerItem<'a, TLayerCtx, TMiddleware, TNextLayer>,
        >,
        future::Ready<LayerItem<'a, TLayerCtx, TMiddleware, TNextLayer>>,
    >;

    type MwResult<TLayerCtx, TMiddleware> = <TMiddleware as SealedMiddleware<TLayerCtx>>::Result;
    type MwFut<TLayerCtx, TMiddleware> = <TMiddleware as SealedMiddleware<TLayerCtx>>::Fut;

    type RespFut<TLayerCtx, TMiddleware> =
        <<MwResult<TLayerCtx, TMiddleware> as MwV2Result>::Resp as Executable2>::Fut;
    type RespFn<TLayerCtx, TMiddleware> =
        Option<<MwResult<TLayerCtx, TMiddleware> as MwV2Result>::Resp>;

    type Bruh2<'a, TLayerCtx, TMiddleware, TNextLayer> = stream::Then<
        stream::Zip<
            LayerStream<'a, TLayerCtx, TMiddleware, TNextLayer>,
            stream::Repeat<RespFn<TLayerCtx, TMiddleware>>,
        >,
        Bruh<'a, TLayerCtx, TMiddleware, TNextLayer>,
        fn(
            (
                LayerItem<'a, TLayerCtx, TMiddleware, TNextLayer>,
                RespFn<TLayerCtx, TMiddleware>,
            ),
        ) -> Bruh<'a, TLayerCtx, TMiddleware, TNextLayer>,
    >;

    type FlatMapContents<'a, TLayerCtx, TMiddleware, TNextLayer> = future::Either<
        Bruh2<'a, TLayerCtx, TMiddleware, TNextLayer>,
        stream::Once<future::Ready<Result<Value, ExecError>>>,
    >;

    fn bruh2<
        'a,
        TLayerCtx: SendSyncStatic,
        TMiddleware: Middleware<TLayerCtx>,
        TNextLayer: Layer<TMiddleware::NewCtx>,
    >(
        stream: TNextLayer::Stream<'a>,
        resp_fn: RespFn<TLayerCtx, TMiddleware>,
    ) -> Bruh2<'a, TLayerCtx, TMiddleware, TNextLayer> {
        stream
            .zip(stream::repeat(resp_fn))
            .then(|(result, resp_fn)| match resp_fn {
                Some(resp_fn) => match result {
                    Ok(result) => resp_fn.call(result).map(Ok as fn(_) -> _).left_future(),
                    Err(err) => future::ready(Err(err)).right_future(),
                },
                None => future::ready(result).right_future(),
            })
    }

    fn inner<
        'a,
        TLayerCtx: SendSyncStatic,
        TMiddleware: Middleware<TLayerCtx>,
        TNextLayer: Layer<TMiddleware::NewCtx>,
    >(
        (fut, next): (TMiddleware::Result, &'a TNextLayer),
    ) -> FlatMapContents<'a, TLayerCtx, TMiddleware, TNextLayer> {
        fn inner<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        >(
            ((ctx, input, req, resp_fn), next): (
                ExplodedMwResult<TMiddleware::Result>,
                &'a TNextLayer,
            ),
        ) -> Result<
            (
                TNextLayer::Stream<'a>,
                Option<<TMiddleware::Result as MwV2Result>::Resp>,
            ),
            ExecError,
        > {
            next.call(ctx, input, req).map(|stream| (stream, resp_fn))
        }

        let (stream, resp_fn) = match fut
            .explode()
            .map(|v| (v, next))
            .and_then(inner::<TLayerCtx, TMiddleware, TNextLayer>)
        {
            Ok(v) => v,
            Err(e) => {
                return stream::once(future::ready(Err(e))).right_stream::<Bruh2<
                    'a,
                    TLayerCtx,
                    TMiddleware,
                    TNextLayer,
                >>()
            }
        };

        bruh2::<'a, TLayerCtx, TMiddleware, TNextLayer>(stream, resp_fn).left_stream()
    }

    type FlatMapFromFuture<TFuture, TRet> =
        stream::FlatMap<future::IntoStream<TFuture>, TRet, fn(<TFuture as Future>::Output) -> TRet>;

    type MiddlewareLayerFutureNew<'a, TLayerCtx, TMiddleware, TNextLayer> = FlatMapFromFuture<
        future::Join<MwFut<TLayerCtx, TMiddleware>, future::Ready<&'a TNextLayer>>,
        FlatMapContents<'a, TLayerCtx, TMiddleware, TNextLayer>,
    >;

    pin_project! {
        pub struct MiddlewareLayerFuture<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > {
            #[pin]
            inner: MiddlewareLayerFutureNew<'a, TLayerCtx, TMiddleware, TNextLayer>
        }
    }

    impl<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx> + Send,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > Stream for MiddlewareLayerFuture<'a, TLayerCtx, TMiddleware, TNextLayer>
    {
        type Item = Result<Value, ExecError>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.project().inner.poll_next(cx)
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.inner.size_hint()
        }
    }

    impl<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx> + Send,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > Body for MiddlewareLayerFuture<'a, TLayerCtx, TMiddleware, TNextLayer>
    {
    }

    fn middleware_layer_future<
        'a,
        TLayerCtx: SendSyncStatic,
        TMiddleware: Middleware<TLayerCtx> + Send,
        TNextLayer: Layer<TMiddleware::NewCtx>,
    >(
        fut: TMiddleware::Fut,
        next: &'a TNextLayer,
    ) -> MiddlewareLayerFutureNew<'a, TLayerCtx, TMiddleware, TNextLayer> {
        future::join(fut, future::ready(next))
            .into_stream()
            .flat_map(inner::<TLayerCtx, TMiddleware, TNextLayer>)
    }
}

pub(crate) use private::MiddlewareLayer;
