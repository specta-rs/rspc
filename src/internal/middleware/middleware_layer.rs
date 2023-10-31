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
        cursed::{self, YieldMsg, CURSED_OP},
        error::ExecError,
        internal::{
            new_mw_ctx, IntoMiddlewareResult, Layer, PinnedOption, PinnedOptionProj, ProcedureDef,
            RequestContext,
        },
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
            let fut = self.mw.execute(
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
                stream: PinnedOption::None,
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

                // The actual data stream from the resolver function or next middleware
                #[pin]
                stream: PinnedOption<TNextLayer::Stream<'a>>,
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
                        mut stream,
                    } => {
                        // TODO: We need to call the underlying stream and setup waker or pipe value to middleware's next value

                        // TODO: Handle `is_done`
                        if let PinnedOptionProj::Some { v } = stream.as_mut().project() {
                            match v.poll_next(cx) {
                                Poll::Ready(Some(v)) => {
                                    println!("{v:?}");
                                    CURSED_OP.set(Some(YieldMsg::YieldBodyResult(
                                        match v.unwrap() {
                                            ValueOrBytes::Value(v) => v,
                                            _ => todo!(),
                                        },
                                    )));
                                }
                                Poll::Ready(None) => {
                                    // TODO: Don't do this and instead stop internally and keep user's future running but this works for now
                                    self.as_mut().set(Self::PendingDone);
                                    return Poll::Ready(None);
                                }
                                Poll::Pending => return Poll::Pending, // TODO: `return` if underlying stream is waiting for a value
                            }
                        }

                        match fut.poll(cx) {
                            Poll::Ready(result) => {
                                self.as_mut().set(Self::PendingDone);
                                return Poll::Ready(Some(
                                    result.into_result().map(ValueOrBytes::Value),
                                ));
                            }
                            Poll::Pending => {
                                // cursed::outer(cx.waker());

                                if let Some(op) = CURSED_OP.take() {
                                    match op {
                                        YieldMsg::YieldBody => {
                                            // TODO: Value from thread_local system instead to avoid `Arc`???
                                            let ctx = new_ctx
                                                .lock()
                                                .unwrap_or_else(PoisonError::into_inner)
                                                .take()
                                                .unwrap();

                                            match next.layer_call(
                                                ctx,
                                                input.take().unwrap(),
                                                req.take().unwrap(),
                                            ) {
                                                Ok(sstream) => {
                                                    println!("SET SOME");
                                                    stream.set(PinnedOption::Some { v: sstream });
                                                    continue;
                                                }

                                                Err(err) => {
                                                    cx.waker().wake_by_ref(); // No wakers set so we set one
                                                    self.as_mut().set(Self::PendingDone);
                                                    return Poll::Ready(Some(Err(err)));
                                                }
                                            }
                                        }
                                        YieldMsg::YieldBodyResult(_) => unreachable!(),
                                    }
                                }

                                return Poll::Pending;
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
                // Self::Execute { stream: c, .. } => c.size_hint(), // TODO: Bring this back
                _ => (0, None),
            }
        }
    }
}

pub(crate) use private::MiddlewareLayer;
