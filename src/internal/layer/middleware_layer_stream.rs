use std::{
    cell::Cell,
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::FusedFuture, stream::FusedStream, Stream};

enum YieldMsg {
    PlzYieldChunk,
    YieldedChunk(Option<serde_json::Value>),
}

thread_local! {
    // TODO: Explain this crazy shit.
    static OPERATION: Cell<Option<YieldMsg>> = const { Cell::new(None) };
}

pub struct MiddlewareLayerStream<S>(S);

impl<S: Stream> Stream for MiddlewareLayerStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // TODO: Safety
            let inner = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0) };
            match inner.poll_next(cx) {
                Poll::Ready(result) => return Poll::Ready(result),
                Poll::Pending => {
                    if let Some(op) = OPERATION.take() {
                        match op {
                            YieldMsg::PlzYieldChunk => {
                                OPERATION.set(Some(YieldMsg::YieldedChunk(None))); // TODO: Poll inner stream for `Value` instead.
                                continue; // We re-poll the inner stream and so it receives the chunk.
                            }
                            YieldMsg::YieldedChunk(_) => unreachable!(),
                        }
                    }

                    return Poll::Pending;
                }
            }
        }
    }
}

// TODO: Rename
pub struct NextStream {
    yielded: bool,
    done: bool,
}

impl NextStream {
    pub(crate) fn new() -> Self {
        Self {
            yielded: false,
            done: false,
        }
    }
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
