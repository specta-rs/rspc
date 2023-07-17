mod private {
    use std::{
        marker::PhantomData,
        pin::Pin,
        task::{ready, Context, Poll},
    };

    use futures::{Future, Stream};
    use pin_project::pin_project;
    use serde_json::Value;

    use crate::{
        internal::{
            middleware::Middleware,
            middleware::{Executable2, MiddlewareContext, MwV2Result, RequestContext},
            Layer, SealedLayer,
        },
        ExecError,
    };

    #[doc(hidden)]
    pub struct MiddlewareLayer<TLayerCtx, TMiddleware, TNewMiddleware> {
        pub(crate) next: TMiddleware,
        pub(crate) mw: TNewMiddleware,
        pub(crate) phantom: PhantomData<TLayerCtx>,
    }

    impl<TLayerCtx, TMiddleware, TNewMiddleware> SealedLayer<TLayerCtx>
        for MiddlewareLayer<TLayerCtx, TMiddleware, TNewMiddleware>
    where
        TLayerCtx: Send + Sync + 'static,
        TMiddleware: Layer<TNewMiddleware::NewCtx> + Sync + 'static,
        TNewMiddleware: Middleware<TLayerCtx> + Send + Sync + 'static,
    {
        type Stream<'a> = MiddlewareLayerFuture<'a, TLayerCtx, TNewMiddleware, TMiddleware>;

        fn call(
            &self,
            ctx: TLayerCtx,
            input: Value,
            req: RequestContext,
        ) -> Result<Self::Stream<'_>, ExecError> {
            let fut = self.mw.run_me(
                ctx,
                MiddlewareContext {
                    input,
                    req,
                    _priv: (),
                },
            );

            Ok(MiddlewareLayerFuture::First {
                fut,
                next: &self.next,
            })
        }
    }

    // TODO: Cleanup generics on this
    // TODO: Document phases
    #[pin_project(project = MiddlewareLayerFutureProj)]
    pub enum MiddlewareLayerFuture<
        'a,
        // TODO: Remove one of these Ctx's and get from `TMiddleware` or `TNextMiddleware`
        TLayerCtx: Send + Sync + 'static,
        TNewMiddleware: Middleware<TLayerCtx>,
        TMiddleware: Layer<TNewMiddleware::NewCtx>,
    > {
        First {
            #[pin]
            fut: TNewMiddleware::Fut,
            next: &'a TMiddleware,
        },
        Second {
            #[pin]
            stream: TMiddleware::Stream<'a>,
            resp: Option<<TNewMiddleware::Result as MwV2Result>::Resp>,
        },
        Third {
            #[pin]
            fut: <<TNewMiddleware::Result as MwV2Result>::Resp as Executable2>::Fut,
        },
    }

    impl<
            'a,
            TLayerCtx: Send + Sync + 'static,
            TNewMiddleware: Middleware<TLayerCtx>,
            TMiddleware: Layer<TNewMiddleware::NewCtx>,
        > Stream for MiddlewareLayerFuture<'a, TLayerCtx, TNewMiddleware, TMiddleware>
    {
        type Item = Result<Value, ExecError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            loop {
                let new_value = match self.as_mut().project() {
                    MiddlewareLayerFutureProj::First { fut, next } => {
                        let result = ready!(fut.poll(cx));

                        let (ctx, input, req, resp) = result.explode()?;

                        match next.call(ctx, input, req) {
                            Ok(stream) => Self::Second { stream, resp },
                            Err(err) => return Poll::Ready(Some(Err(err))),
                        }
                    }
                    MiddlewareLayerFutureProj::Second { stream, resp } => {
                        let result = ready!(stream.poll_next(cx));

                        let Some(resp) = resp.take() else {
                        	return Poll::Ready(result);
                        };

                        let result = result.unwrap().unwrap();

                        Self::Third {
                            fut: resp.call(result),
                        }
                    }
                    MiddlewareLayerFutureProj::Third { fut } => {
                        return fut.poll(cx).map(|result| Some(Ok(result)))
                    }
                };

                self.as_mut().set(new_value);
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match &self {
                Self::Second { stream: c, .. } => c.size_hint(),
                _ => (0, None),
            }
        }
    }
}

pub(crate) use private::MiddlewareLayer;
