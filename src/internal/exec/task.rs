use std::{fmt, pin::Pin, sync::Arc, task::Poll};

use futures::{ready, Stream};

use crate::{
    internal::{exec, Body},
    Router,
};

use super::arc_ref::ArcRef;

// TODO: Should this be called `Task` or `StreamWrapper`? Will depend on it's final form.

pub enum Status {
    ShouldBePolled { done: bool },
    DoNotPoll,
}

// TODO: docs
pub struct Task {
    pub(crate) id: u32,
    // You will notice this is a `Stream` not a `Future` like would be implied by the struct.
    // rspc's whole middleware system only uses `Stream`'s cause it makes life easier so we change to & from a `Future` at the start/end.
    pub(crate) stream: ArcRef<Pin<Box<dyn Body + Send>>>,
    // pub(crate) shutdown
    // Mark when the stream is done. This means `self.reference` returned `None` but we still had to yield the complete message so we haven't returned `None` yet.
    pub(crate) status: Status,
}

// pub enum Inner {
//     Task(Task),
//     Response(exec::Response),
// }

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
        match &self.status {
            Status::DoNotPoll => {
                #[cfg(debug_assertions)]
                panic!("`StreamWrapper` polled after completion")
            }
            Status::ShouldBePolled { done } => {
                if *done {
                    self.status = Status::DoNotPoll;
                    return Poll::Ready(None);
                }
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
                cx.waker().wake_by_ref(); // We want the stream to be called again so we can return `None` and close it
                self.status = Status::ShouldBePolled { done: true };
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

// TODO: Unit test
