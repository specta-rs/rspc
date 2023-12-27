use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_core::{FusedStream, Stream};

use crate::Format;

/// TODO
pub struct Task<F> {
    /// The unique identifier of the task.
    /// Be careful storing these as they may be reused after the task is completed.
    pub id: u32,
    /// Whether the task requires queuing.
    /// If this is `false` you can safely block inline until it's done.
    /// For example a streaming query would be `false` while a subscription would be `true`.
    pub requires_queuing: bool,
    // done: bool, // TODO
    // Task must be `'static` so it can be queued onto an async runtime like Tokio.
    // However this borrows from the `Arc` in `Executor`.
    // TODO: Finish the explanation
    stream: Pin<Box<dyn Future<Output = ()> + Send>>,
    format: Arc<F>,
}

impl<F> fmt::Debug for Task<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl<F: Format> Stream for Task<F> {
    type Item = F::Result;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // let serialize = self.format.serializer();

        // TODO: How to get `serialize` into the future?

        match self.stream.as_mut().poll(cx) {
            Poll::Ready(()) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<F: Format> FusedStream for Task<F> {
    fn is_terminated(&self) -> bool {
        todo!()
    }
}
