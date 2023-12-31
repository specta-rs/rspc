//! This file uses `unsafe` so be careful making anything public.

use std::{
    any::Any,
    cell::Cell,
    fmt,
    future::poll_fn,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{stream::FusedStream, Stream};
use serde_json::Value;

use crate::internal::middleware::RequestContext;

enum YieldMsg {
    // The `&'static` lifetime is fake.
    InitInnerStream {
        ctx: &'static mut (dyn Any + Send),
        input: Value,
        req: RequestContext,
    },
    PlzYieldChunk,
    YieldedChunk(Option<serde_json::Value>),
}

thread_local! {
    // TODO: Explain this crazy shit.
    static OPERATION: Cell<Option<YieldMsg>> = const { Cell::new(None) };
}

pub(crate) enum OnPendingAction {
    Pending,
    Continue,
}

// TODO: Rename
pub(crate) enum MiddlewareStreamIntersector<TNewCtx, F, S>
where
    TNewCtx: 'static,
    F: FnOnce(TNewCtx, Value, RequestContext) -> S,
    S: Stream + 'static,
{
    WaitingInit(F),
    PollingStream(S),
    Done,
    PhantomData(PhantomData<TNewCtx>),
}

impl<TNewCtx, F, S> MiddlewareStreamIntersector<TNewCtx, F, S>
where
    TNewCtx: 'static,
    F: FnOnce(TNewCtx, Value, RequestContext) -> S,
    S: Stream + 'static,
{
    pub fn on_pending(&mut self) -> OnPendingAction {
        if let Some(op) = OPERATION.take() {
            match op {
                // Holding onto `next_ctx` would be unsafe as it points to a stack value so we take it as early as possible.
                YieldMsg::InitInnerStream { ctx, input, req } => {
                    let next_ctx: &mut Option<TNewCtx> = ctx.downcast_mut().unwrap(); // TODO: Error handling
                    let next_ctx = next_ctx.take().unwrap(); // TODO: Error handling

                    match self {
                        MiddlewareStreamIntersector::WaitingInit(stream_fn) => {
                            // *self = MiddlewareStreamIntersector::PollingStream((stream_fn)(
                            //     next_ctx, input, req,
                            // ));
                            todo!();
                        }
                        MiddlewareStreamIntersector::PollingStream(_) => unreachable!(),
                        MiddlewareStreamIntersector::Done => unreachable!(),
                        MiddlewareStreamIntersector::PhantomData(_) => unreachable!(),
                    }
                }
                YieldMsg::PlzYieldChunk => {
                    OPERATION.set(Some(YieldMsg::YieldedChunk(None))); // TODO: Poll inner stream for `Value` instead.
                    return OnPendingAction::Continue;
                }
                YieldMsg::YieldedChunk(_) => unreachable!(),
            }
        }

        OnPendingAction::Pending
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
                YieldMsg::PlzYieldChunk => unreachable!(),
                YieldMsg::YieldedChunk(chunk) => {
                    self.done = chunk.is_none();
                    Poll::Ready(chunk)
                }
            }
        } else {
            self.yielded = true;
            OPERATION.set(Some(YieldMsg::PlzYieldChunk));
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
