mod private {
    use std::{
        borrow::Cow,
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    use futures::{
        future::{self, ok, ready, Ready},
        stream::{self, once, Once},
        Future, FutureExt, Stream, StreamExt,
    };
    use pin_project_lite::pin_project;
    use serde_json::Value;
    use specta::{ts, TypeMap};

    use crate::internal::middleware::Middleware;
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
        TLayerCtx: Send + Sync + 'static,
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

            Ok(MiddlewareLayerFuture::Resolve {
                fut,
                next: &self.next,
            })
        }
    }

    // This exists because `pin_project_lite` doesn't understand `+` bounds
    pub trait SendSyncStatic: Send + Sync + 'static {}
    impl<T: Send + Sync + 'static> SendSyncStatic for T {}

    pin_project! {
        #[project = MiddlewareLayerFutureProj]
        pub enum MiddlewareLayerFuture<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > {
            // We are waiting for the current middleware to run and yield it's result.
            // Remember the middleware only runs once for an entire stream as it returns "instructions" on how to map the stream from then on.
            Resolve {
                // Future of the currently middleware.
                // It's result will populate the `resp_fn` field for the next phase.
                #[pin]
                fut: TMiddleware::Fut,

                // The next layer in the middleware chain
                // This could be another middleware of the users resolver. It will be called to yield the `stream` for the next phase.
                next: &'a TNextLayer,
            },
            // We are in this state where we are executing the current middleware on the stream
            Execute {
                // The actual data stream from the resolver function or next middleware
                #[pin]
                stream: TNextLayer::Stream<'a>,
                // We use this so we can keep polling `resp_fut` for the final message and once it is done and this bool is set, shutdown.
                is_stream_done: bool,

                // The currently executing future returned by the `resp_fn` (publicly `.map`) function
                // Be aware this will go `None` -> `Some` -> `None`, etc for a subscription
                #[pin]
                resp_fut: PinnedOption<<<TMiddleware::Result as MwV2Result>::Resp as Executable2>::Fut>,
                // The `.map` function returned by the user from the execution of the current middleware
                // This allows a middleware to map the values being returned from the stream
                resp_fn: Option<<TMiddleware::Result as MwV2Result>::Resp>,
            },
            // The stream is internally done but it returned `Poll::Ready` for the shutdown message so the caller thinks it's still active
            // This will yield `Poll::Ready(None)` and transition into the `Self::Done` phase.
            PendingDone,
            // Stream is completed and will panic if polled again
            Done,
        }
    }

    impl<
            'a,
            TLayerCtx: Send + Sync + 'static,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > Body for MiddlewareLayerFuture<'a, TLayerCtx, TMiddleware, TNextLayer>
    {
    }

    impl<
            'a,
            TLayerCtx: Send + Sync + 'static,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > Stream for MiddlewareLayerFuture<'a, TLayerCtx, TMiddleware, TNextLayer>
    {
        type Item = Result<Value, ExecError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            loop {
                match self.as_mut().project() {
                    MiddlewareLayerFutureProj::Resolve { fut, next } => {
                        let result = futures::ready!(fut.poll(cx));
                        let (ctx, input, req, resp_fn) = match result.explode() {
                            Ok(v) => v,
                            Err(err) => {
                                cx.waker().wake_by_ref(); // No wakers set so we set one
                                self.as_mut().set(Self::PendingDone);
                                return Poll::Ready(Some(Err(err)));
                            }
                        };

                        match next.call(ctx, input, req) {
                            Ok(stream) => {
                                self.as_mut().set(Self::Execute {
                                    stream,
                                    is_stream_done: false,
                                    resp_fut: PinnedOption::None,
                                    resp_fn,
                                });
                            }

                            Err(err) => {
                                cx.waker().wake_by_ref(); // No wakers set so we set one
                                self.as_mut().set(Self::PendingDone);
                                return Poll::Ready(Some(Err(err)));
                            }
                        }
                    }
                    MiddlewareLayerFutureProj::Execute {
                        mut stream,
                        is_stream_done,
                        mut resp_fut,
                        resp_fn,
                    } => {
                        if let PinnedOptionProj::Some { v } = resp_fut.as_mut().project() {
                            let result = futures::ready!(v.poll(cx));
                            cx.waker().wake_by_ref(); // No wakers set so we set one
                            resp_fut.set(PinnedOption::None);
                            return Poll::Ready(Some(Ok(result)));
                        }

                        if *is_stream_done {
                            self.as_mut().set(Self::Done);
                            return Poll::Ready(None);
                        }

                        match futures::ready!(stream.as_mut().poll_next(cx)) {
                            Some(result) => match resp_fn {
                                Some(resp_fn) => match result {
                                    Ok(result) => {
                                        resp_fut.set(PinnedOption::Some {
                                            v: (*resp_fn).call(result),
                                        });
                                        continue;
                                    }
                                    // TODO: The `.map` function is skipped for errors. Maybe it should be possible to map them when desired?
                                    // TODO: We also shut down the whole stream on a single error. Is this desired?
                                    Err(err) => {
                                        cx.waker().wake_by_ref(); // No wakers set so we set one
                                        self.as_mut().set(Self::PendingDone);
                                        return Poll::Ready(Some(Err(err)));
                                    }
                                },

                                // No `.map` fn so we return the result as is
                                None => {
                                    cx.waker().wake_by_ref(); // No wakers set so we set one
                                    return Poll::Ready(Some(result));
                                }
                            },
                            // The underlying stream has shutdown so we will resolve `resp_fut` and then terminate ourselves
                            None => {
                                *is_stream_done = true;
                                continue;
                            }
                        }
                    }
                    MiddlewareLayerFutureProj::PendingDone => {
                        println!("PENDING DONE");
                        self.as_mut().set(Self::Done);
                        return Poll::Ready(None);
                    }
                    #[allow(clippy::panic)]
                    MiddlewareLayerFutureProj::Done => {
                        #[cfg(debug_assertions)]
                        panic!("`MiddlewareLayerFuture` polled after completion");

                        #[cfg(not(debug_assertions))]
                        return Poll::Ready(None);
                    }
                }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match &self {
                Self::Execute { stream: c, .. } => c.size_hint(),
                _ => (0, None),
            }
        }
    }

    async fn middleware_layer_future<
        'a,
        TLayerCtx: SendSyncStatic,
        TMiddleware: Middleware<TLayerCtx>,
        TNextLayer: Layer<TMiddleware::NewCtx>,
    >(
        fut: TMiddleware::Fut,
        next: &'a TNextLayer,
    ) -> impl Stream<Item = Result<Value, ExecError>> + 'a {
        type Bruh<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > = future::Either<
            future::Map<
                RespFut<TLayerCtx, TMiddleware>,
                fn(Value) -> <TNextLayer::Stream<'a> as Stream>::Item,
            >,
            future::Ready<<TNextLayer::Stream<'a> as Stream>::Item>,
        >;

        type RespFut<TLayerCtx: SendSyncStatic, TMiddleware: Middleware<TLayerCtx>> =
            <<TMiddleware::Result as MwV2Result>::Resp as Executable2>::Fut;
        type RespFn<TLayerCtx: SendSyncStatic, TMiddleware: Middleware<TLayerCtx>> =
            Option<<TMiddleware::Result as MwV2Result>::Resp>;

        type Bruh2<
            'a,
            TLayerCtx: SendSyncStatic,
            TMiddleware: Middleware<TLayerCtx>,
            TNextLayer: Layer<TMiddleware::NewCtx>,
        > = stream::Then<
            stream::Zip<TNextLayer::Stream<'a>, stream::Repeat<RespFn<TLayerCtx, TMiddleware>>>,
            Bruh<'a, TLayerCtx, TMiddleware, TNextLayer>,
            fn(
                (
                    <TNextLayer::Stream<'a> as Stream>::Item,
                    RespFn<TLayerCtx, TMiddleware>,
                ),
            ) -> Bruh<'a, TLayerCtx, TMiddleware, TNextLayer>,
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
        ) -> future::Either<
            Bruh2<'a, TLayerCtx, TMiddleware, TNextLayer>,
            Once<Ready<Result<Value, ExecError>>>,
        > {
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

        fut.into_stream().flat_map(|fut| {
            let (stream, resp_fn) = match fut.explode().and_then(|(ctx, input, req, resp_fn)| {
                next.call(ctx, input, req).map(|stream| (stream, resp_fn))
            }) {
                Ok(v) => v,
                Err(e) => return stream::iter([Err(e)]).right_stream(),
            };

            stream
                .then(move |result| match &resp_fn {
                    Some(resp_fn) => match result {
                        Ok(result) => resp_fn.call(result).map(Ok).left_future(),
                        Err(err) => future::ready(Err(err)).right_future(),
                    },
                    None => future::ready(result).right_future(),
                })
                .left_stream()
        })
    }
}

pub(crate) use private::MiddlewareLayer;
