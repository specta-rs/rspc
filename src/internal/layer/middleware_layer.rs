use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc, task::Poll};

use futures::{future::Either, Stream};
use serde_json::Value;

use crate::{
    error::ExecError,
    internal::middleware::{new_mw_ctx, IntoMiddlewareResult, MiddlewareFn, RequestContext},
};

use super::Layer;

#[doc(hidden)]
pub struct MiddlewareLayer<TLayerCtx, TNewCtx, TNextMiddleware, TNewMiddleware> {
    // TODO: This `Arc` saves us a hole load of pain.
    pub(crate) next: Arc<TNextMiddleware>,
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
    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<impl Stream<Item = Result<Value, ExecError>> + Send + 'static, ExecError> {
        let mut state = Either::Left(self.mw.execute(ctx, new_mw_ctx(input, req)));
        let mut done = false;
        // let mut intersector =
        //     MiddlewareStreamIntersector::<TNewCtx, _, _>::WaitingInit(|ctx, input, req| {
        //         self.next.call(ctx, input, req).unwrap() // TODO: Error handling
        //     });

        Ok(futures::stream::poll_fn(move |cx| {
            // let intersector = &mut intersector;

            loop {
                if done {
                    return Poll::Ready(None);
                }

                match &mut state {
                    // Poll the middleware future
                    Either::Left(fut) => {
                        let fut = unsafe { Pin::new_unchecked(fut) };
                        match fut.poll(cx) {
                            Poll::Ready(result) => match result.into_result() {
                                Ok(result) => {
                                    state = Either::Right(result);
                                    continue;
                                }
                                Err(err) => {
                                    done = true;
                                    return Poll::Ready(Some(Err(err)));
                                }
                            },
                            Poll::Pending => {
                                // let _ = interseptor.on_pending();

                                // let y = self.next.call(ctx, input, req);

                                // match on_pending(cx, || fut.as_mut().poll(cx)) {
                                //     OnPendingAction::Continue => continue,
                                //     OnPendingAction::Pending => return Poll::Pending,
                                // }
                                todo!();
                            }
                        }
                    }
                    // Poll the middleware stream. This potentially be returned from the middleware future.
                    Either::Right(stream) => {
                        let stream = unsafe { Pin::new_unchecked(stream) };
                        return match stream.poll_next(cx) {
                            Poll::Ready(None) => {
                                done = true;
                                Poll::Ready(None)
                            }
                            Poll::Ready(v) => Poll::Ready(v),
                            Poll::Pending => {
                                // match on_pending() {
                                //     OnPendingAction::Continue => continue,
                                //     OnPendingAction::Pending => Poll::Pending,
                                // }
                                todo!();
                            }
                        };
                    }
                }
            }
        }))
    }
}
