use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::{FusedStream, Stream};

pub struct Task {
    pub id: u32,
    // pub should_be_queued: bool,
    // done: bool,
    stream: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl Stream for Task {
    type Item = Vec<u8>; // TODO: What if the user wants `serde_json::Value`

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

impl FusedStream for Task {
    fn is_terminated(&self) -> bool {
        todo!()
    }
}
