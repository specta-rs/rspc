mod private {
    use std::{
        borrow::Cow,
        marker::PhantomData,
        pin::Pin,
        sync::{Arc, Mutex, PoisonError},
        task::{ready, Context, Poll},
    };

    use futures::{Future, Stream};
    use pin_project_lite::pin_project;
    use serde_json::Value;
    use specta::{ts, TypeMap};

    use rspc_core::{
        error::ExecError,
        internal::{new_mw_ctx, Layer, ProcedureDef, RequestContext},
        ValueOrBytes,
    };

    use crate::internal::middleware::MiddlewareFn;

    #[doc(hidden)]
    pub struct MiddlewareLayer<TLayerCtx, TNewCtx, TNextLayer, TNewMiddleware> {
        pub(crate) next: TNextLayer,
        pub(crate) mw: TNewMiddleware,
        pub(crate) phantom: PhantomData<(TLayerCtx, TNewCtx)>,
    }

    impl<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware> Layer<TLayerCtx>
        for MiddlewareLayer<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware>
    where
        TLayerCtx: Send + Sync + 'static,
        TNewCtx: Send + Sync + 'static,
        TNextMiddleware: Layer<TNewCtx> + Sync + 'static,
        TNewMiddleware: MiddlewareFn<TLayerCtx, TNewCtx> + Send + Sync + 'static,
    {
        type Stream<'a> =
            MiddlewareLayerFuture<'a, TNewCtx, TLayerCtx, TNewMiddleware, TNextMiddleware>;

        fn into_procedure_def(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, ts::ExportError> {
            self.next.into_procedure_def(key, ty_store)
        }

        fn layer_call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
        ) -> Result<Self::Stream<'_>, ExecError> {
            let new_ctx = Arc::new(Mutex::new(None));
            let fut = self.mw.run_me(
                ctx,
                new_mw_ctx(
                    input.clone(), // TODO: This probs won't fly if we accept file upload
                    req.clone(),
                    new_ctx.clone(),
                ),
            );

            Ok(MiddlewareLayerFuture::Resolve {
                fut,
                next: &self.next,
                new_ctx,
                input: Some(input),
                req: Some(req),
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
            TNewCtx: 'static,
            TLayerCtx: SendSyncStatic,
            TMiddleware: MiddlewareFn<TLayerCtx, TNewCtx>,
            TNextLayer: Layer<TNewCtx>,
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

                // TODO
                new_ctx: Arc<Mutex<Option<TNewCtx>>>,

                // TODO: Avoid `Option` and instead encode into enum
                input: Option<Value>,
                req: Option<RequestContext>,
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
                // #[pin]
                // resp_fut: PinnedOption<<<TMiddleware::Result as MwV2Result>::Resp as Executable2>::Fut>,
                // The `.map` function returned by the user from the execution of the current middleware
                // This allows a middleware to map the values being returned from the stream
                // resp_fn: <TMiddleware::Result as MwV2Result>::Resp,
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
            TNewCtx: 'static,
            TLayerCtx: Send + Sync + 'static,
            TMiddleware: MiddlewareFn<TLayerCtx, TNewCtx>,
            TNextLayer: Layer<TNewCtx>,
        > Stream for MiddlewareLayerFuture<'a, TNewCtx, TLayerCtx, TMiddleware, TNextLayer>
    {
        type Item = Result<ValueOrBytes, ExecError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            loop {
                match self.as_mut().project() {
                    MiddlewareLayerFutureProj::Resolve {
                        fut,
                        next,
                        new_ctx,
                        input,
                        req,
                    } => {
                        let result = ready!(fut.poll(cx));

                        let ctx = new_ctx
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .take()
                            .unwrap();

                        match next.call(ctx, input.take().unwrap(), req.take().unwrap()) {
                            Ok(stream) => {
                                self.as_mut().set(Self::Execute {
                                    stream,
                                    is_stream_done: false,
                                    // resp_fut: PinnedOption::None,
                                    // resp_fn: None, // TODO: Fully remove this
                                });
                            }

                            Err(err) => {
                                self.as_mut().set(Self::PendingDone);
                                return Poll::Ready(Some(Err(err)));
                            }
                        }
                    }
                    MiddlewareLayerFutureProj::Execute {
                        mut stream,
                        is_stream_done,
                    } => {
                        // if let PinnedOptionProj::Some { v } = resp_fut.as_mut().project() {
                        //     let result = ready!(v.poll(cx));
                        //     cx.waker().wake_by_ref(); // No wakers set so we set one
                        //     resp_fut.set(PinnedOption::None);
                        //     return Poll::Ready(Some(Ok(result)));
                        // }

                        if *is_stream_done {
                            self.as_mut().set(Self::Done);
                            return Poll::Ready(None);
                        }

                        match ready!(stream.as_mut().poll_next(cx)) {
                            Some(result) => match result {
                                Ok(result) => {
                                    return Poll::Ready(Some(Ok(result)));
                                }
                                // TODO: The `.map` function is skipped for errors. Maybe it should be possible to map them when desired?
                                // TODO: We also shut down the whole stream on a single error. Is this desired?
                                Err(err) => {
                                    self.as_mut().set(Self::PendingDone);
                                    return Poll::Ready(Some(Err(err)));
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
}

pub(crate) use private::MiddlewareLayer;
