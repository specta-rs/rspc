mod private {
    use std::{
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    use futures::{Future, Stream};
    use pin_project::pin_project;
    use serde_json::Value;

    use crate::{
        internal::{
            middleware::Middleware,
            middleware::{Executable2, MiddlewareContext, MwV2Result, RequestContext},
            Layer, PinnedOption, PinnedOptionProj, SealedLayer,
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

            Ok(MiddlewareLayerFuture {
                a: PinnedOption::Some(fut),
                b: &self.next,
                c: PinnedOption::None,
                d: None,
                e: PinnedOption::None,
            })
        }
    }

    // TODO: Cleanup generics on this
    // TODO: Fix potential panic in this
    #[pin_project(project = MiddlewareLayerFutureProj)]
    pub struct MiddlewareLayerFuture<
        'a,
        // TODO: Remove one of these Ctx's and get from `TMiddleware` or `TNextMiddleware`
        TLayerCtx: Send + Sync + 'static,
        TNewMiddleware: Middleware<TLayerCtx> + Send + Sync + 'static,
        TMiddleware: Layer<TNewMiddleware::NewCtx> + 'static,
    > {
        #[pin]
        a: PinnedOption<TNewMiddleware::Fut>,
        b: &'a TMiddleware,
        #[pin]
        c: PinnedOption<TMiddleware::Stream<'a>>,
        d: Option<<TNewMiddleware::Result as MwV2Result>::Resp>,
        #[pin]
        e: PinnedOption<<<TNewMiddleware::Result as MwV2Result>::Resp as Executable2>::Fut>,
    }

    impl<
            'a,
            TLayerCtx: Send + Sync + 'static,
            TNewMiddleware: Middleware<TLayerCtx> + Send + Sync + 'static,
            TMiddleware: Layer<TNewMiddleware::NewCtx> + 'static,
        > Stream for MiddlewareLayerFuture<'a, TLayerCtx, TNewMiddleware, TMiddleware>
    {
        type Item = Result<Value, ExecError>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let mut this = self.project();

            match this.a.as_mut().project() {
                PinnedOptionProj::Some(fut) => match fut.poll(cx) {
                    Poll::Ready(result) => {
                        this.a.set(PinnedOption::None);

                        let (ctx, input, req, resp) = result.explode()?;
                        *this.d = resp;

                        match this.b.call(ctx, input, req) {
                            Ok(stream) => this.c.set(PinnedOption::Some(stream)),
                            Err(err) => return Poll::Ready(Some(Err(err))),
                        }
                    }
                    Poll::Pending => return Poll::Pending,
                },
                PinnedOptionProj::None => {}
            }

            match this.e.as_mut().project() {
                PinnedOptionProj::Some(fut) => match fut.poll(cx) {
                    Poll::Ready(result) => {
                        this.e.set(PinnedOption::None);

                        return Poll::Ready(Some(Ok(result)));
                    }
                    Poll::Pending => return Poll::Pending,
                },
                PinnedOptionProj::None => {}
            }

            match this.c.as_mut().project() {
                PinnedOptionProj::Some(fut) => {
                    match fut.poll_next(cx) {
                        Poll::Ready(result) => {
                            match this.d.take() {
                                Some(resp) => {
                                    // TODO: Deal with this -> The `resp` handler should probs take in the whole `Result`?
                                    let result = result.unwrap().unwrap();

                                    let fut = resp.call(result);
                                    this.e.set(PinnedOption::Some(fut));
                                }
                                None => return Poll::Ready(result),
                            }
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                PinnedOptionProj::None => {}
            }

            unreachable!()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match &self.c {
                PinnedOption::Some(stream) => stream.size_hint(),
                PinnedOption::None => (0, None),
            }
        }
    }
}

pub(crate) use private::MiddlewareLayer;
