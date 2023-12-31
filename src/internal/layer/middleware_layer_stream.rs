//! This file uses `unsafe` so be careful making anything public.

use std::{
    any::Any,
    cell::Cell,
    fmt,
    future::poll_fn,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{stream::FusedStream, Stream};
use serde_json::Value;

use crate::{error::ExecError, internal::middleware::RequestContext};

use super::Layer;

enum YieldMsg {
    // The `&'static` lifetime is fake.
    InitInnerStream {
        ctx: &'static mut (dyn Any + Send),
        input: Value,
        req: RequestContext,
    },
    YieldBodyChunk,
    YieldedChunk(Option<serde_json::Value>),
}

thread_local! {
    // TODO: Explain this crazy shit.
    static OPERATION: Cell<Option<YieldMsg>> = const { Cell::new(None) };
}

pub struct MiddlewareInterceptor<S, TNextMiddleware, TNewCtx>
where
    TNextMiddleware: Layer<TNewCtx>,
    TNewCtx: 'static,
{
    pub(crate) mw: S,
    pub(crate) next: Arc<TNextMiddleware>,
    pub(crate) stream: Option<TNextMiddleware::Stream>,
    pub(crate) phantom: PhantomData<TNewCtx>,
}

impl<S, TNextMiddleware, TNewCtx> Stream for MiddlewareInterceptor<S, TNextMiddleware, TNewCtx>
where
    S: Stream<Item = Result<Value, ExecError>>,
    TNextMiddleware: Layer<TNewCtx>,
    TNewCtx: 'static,
{
    type Item = Result<Value, ExecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let result = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.mw) }
                .as_mut()
                .poll_next(cx);

            if let Poll::Pending = result {
                if let Some(op) = OPERATION.take() {
                    match op {
                        // Holding onto `next_ctx` would be unsafe as it points to a stack value so we take it as early as possible.
                        YieldMsg::InitInnerStream { ctx, input, req } => {
                            let ctx: &mut Option<TNewCtx> = ctx.downcast_mut().unwrap(); // TODO: Error handling
                            let ctx = ctx.take().unwrap(); // TODO: Error handling

                            let stream = self.next.call(ctx, input, req).unwrap(); // TODO: Error handling
                            unsafe { self.as_mut().get_unchecked_mut() }.stream = Some(stream);
                            continue; // Re-poll the middleware or it will stall
                        }
                        YieldMsg::YieldBodyChunk => {
                            let result = if let Some(stream) =
                                &mut unsafe { self.as_mut().get_unchecked_mut() }.stream
                            {
                                let stream = unsafe { Pin::new_unchecked(stream) };
                                match stream.poll_next(cx) {
                                    Poll::Ready(v) => v.map(|v| v.unwrap()), // TODO: Error handling
                                    Poll::Pending => todo!(), // return Poll::Pending, // TODO: We need to know that the stream must be re-polled on the next ready value.
                                }
                            } else {
                                None
                            };

                            OPERATION.set(Some(YieldMsg::YieldedChunk(result)));
                            continue; // Re-poll the middleware or it will stall
                        }
                        YieldMsg::YieldedChunk(_) => unreachable!(),
                    }
                }
            }

            return result;
        }
    }
}

pub async fn get_next_stream<TNewCtx: Send + 'static>(
    new_ctx: TNewCtx,
    input: Value,
    req: RequestContext,
) -> NextStream {
    let new_ctx: &mut (dyn Any + Send) = &mut Some(new_ctx);
    let new_ctx: &'static mut (dyn Any + Send) = unsafe { std::mem::transmute(new_ctx) };
    let mut values = Some((new_ctx, input, req));
    let mut done = false;
    poll_fn(move |_| {
        if done {
            return Poll::Ready(());
        }

        let (ctx, input, req) = values.take().expect("this code path will only be hit once");
        OPERATION.set(Some(YieldMsg::InitInnerStream { ctx, input, req }));
        done = true;
        Poll::Pending
    })
    .await;

    // `NextStream` will only work after `YieldMsg::InitInnerStream` so we return it here to enforce that.
    NextStream {
        yielded: false,
        done: false,
    }
}

// TODO: Rename
pub struct NextStream {
    yielded: bool,
    done: bool,
}

impl fmt::Debug for NextStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NextStream")
            .field("done", &self.done)
            .finish()
    }
}

impl Stream for NextStream {
    // TODO: Should this be `Result<_, ExecError>`???
    type Item = serde_json::Value;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        if self.yielded {
            self.yielded = false;
            let op = OPERATION.take().unwrap();
            match op {
                YieldMsg::InitInnerStream { .. } => unreachable!(),
                YieldMsg::YieldBodyChunk => unreachable!(),
                YieldMsg::YieldedChunk(chunk) => {
                    self.done = chunk.is_none();
                    Poll::Ready(chunk)
                }
            }
        } else {
            self.yielded = true;
            OPERATION.set(Some(YieldMsg::YieldBodyChunk));
            // We don't register a waker. This is okay because it will be re-polled by `MiddlewareLayerStream`.
            Poll::Pending
        }
    }
}

impl FusedStream for NextStream {
    fn is_terminated(&self) -> bool {
        self.done
    }
}
