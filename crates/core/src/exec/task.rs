use std::{fmt, pin::Pin, task::Poll};

use futures::{channel::oneshot, ready, stream::FusedStream, FutureExt, Stream};

use crate::body::Body;
use crate::exec;

use super::{arc_ref::ArcRef, request_future::RequestFuture};

// TODO: Should this be called `Task` or `StreamWrapper`? Will depend on it's final form.

// TODO: docs
pub struct Task {
    pub(crate) id: u32,
    // You will notice this is a `Stream` not a `Future` like would be implied by the struct.
    // rspc's whole middleware system only uses `Stream`'s cause it makes life easier so we change to & from a `Future` at the start/end.
    pub(crate) stream: ArcRef<Pin<Box<dyn Body + Send>>>,
    // Mark when the stream is done. This means `self.reference` returned `None` but we still had to yield the complete message so we haven't returned `None` yet.
    pub(crate) done: bool,
    pub(crate) shutdown_rx: Option<oneshot::Receiver<()>>,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamWrapper")
            .field("id", &self.id)
            .finish()
    }
}

impl Stream for Task {
    type Item = exec::Response;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        if let Some(shutdown_rx) = self.shutdown_rx.as_mut() {
            if shutdown_rx.poll_unpin(cx).is_ready() {
                self.done = true;
            }
        }

        Poll::Ready(Some(match ready!(self.stream.as_mut().poll_next(cx)) {
            Some(r) => exec::Response {
                id: self.id,
                inner: match r {
                    Ok(v) => exec::ResponseInner::Value(v),
                    Err(err) => exec::ResponseInner::Error(err.into()),
                },
            },
            None => {
                let id = self.id;
                self.done = true;
                cx.waker().wake_by_ref();
                exec::Response {
                    id,
                    inner: exec::ResponseInner::Complete,
                }
            }
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.stream.size_hint();
        (min, max.map(|v| v + 1))
    }
}

impl FusedStream for Task {
    fn is_terminated(&self) -> bool {
        self.done
    }
}

impl From<RequestFuture> for Task {
    fn from(value: RequestFuture) -> Self {
        Self {
            id: value.id,
            stream: value.stream,
            done: false,
            shutdown_rx: None,
        }
    }
}

// TODO: Unit test
