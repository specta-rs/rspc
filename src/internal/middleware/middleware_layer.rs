mod private {
    use std::{
        borrow::Cow,
        marker::PhantomData,
        pin::Pin,
        sync::{Arc, Mutex, PoisonError},
        task::{ready, Context, Poll},
    };

    use futures::{channel::oneshot, Future, Stream};
    use pin_project_lite::pin_project;
    use serde_json::Value;
    use specta::{ts, TypeMap};

    use rspc_core::{
        cursed::{self, YieldMsg, CURSED_OP},
        error::ExecError,
        internal::{
            new_mw_ctx, IntoMiddlewareResult, Layer, PinnedOption, ProcedureDef, RequestContext,
        },
        Body, ValueOrBytes,
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
            let (mw_ctx, body_tx) = new_mw_ctx(
                input.clone(), // TODO: This probs won't fly if we accept file upload
                req.clone(),
                new_ctx.clone(),
            );

            let fut = self.mw.execute(ctx, mw_ctx);

            Ok(MiddlewareLayerFuture::Resolve {
                fut,
                next: &self.next,
                new_ctx,
                body_tx: Some(body_tx),
                input: Some(input),
                req: Some(req),
                stream: PinnedOption::None,
                is_stream_done: false,
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

                body_tx: Option<oneshot::Sender<Body>>,

                // TODO: Avoid `Option` and instead encode into enum
                input: Option<Value>,
                req: Option<RequestContext>,

                // The actual data stream from the resolver function or next middleware
                #[pin]
                stream: PinnedOption<TNextLayer::Stream<'a>>,
                // We use this so we can keep polling `resp_fut` for the final message and once it is done and this bool is set, shutdown.
                is_stream_done: bool,
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
                        stream,
                        mut body_tx,
                        is_stream_done,
                    } => {
                        let result = match fut.poll(cx) {
                            Poll::Ready(result) => {
                                self.as_mut().set(Self::PendingDone);
                                return Poll::Ready(Some(
                                    result.into_result().map(ValueOrBytes::Value),
                                ));
                            }
                            Poll::Pending => {
                                if let Some(tx) = body_tx.take() {
                                    tx.send(Body::Value(serde_json::Value::Null)).ok();
                                }

                                return Poll::Pending;
                            }
                        };
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
                // Self::Execute { stream: c, .. } => c.size_hint(), // TODO: Bring this back
                _ => (0, None),
            }
        }
    }
}

pub(crate) use private::MiddlewareLayer;
