use core::fmt;
use std::{
    future::poll_fn,
    pin::Pin,
    task::{Context, Poll},
};

use erased_serde::Serialize;
use futures::Stream;
use pin_project_lite::pin_project;
use serde::Serializer;

/// TODO
// TODO: Rename this type.
pub struct ProcedureStream {
    src: Pin<Box<dyn A>>,
}

impl ProcedureStream {
    pub fn from_stream<S: Stream + 'static>(src: S) -> Self
    where
        S::Item: Serialize, // TODO: Drop this bound!!!
    {
        Self {
            src: Box::pin(StreamToA { src, value: None }),
        }
    }

    // TODO: Would be much better if this was just `next` and the polling was done internally with the same serializer.
    pub fn poll_next<S: Serializer>(
        &mut self,
        cx: &mut Context<'_>,
        serializer: S,
    ) -> Poll<Option<()>> {
        let mut serializer = &mut <dyn erased_serde::Serializer>::erase(serializer);

        self.src.as_mut().poll_next_value(cx, &mut serializer)
    }

    pub async fn next<S: Serializer>(&mut self, serializer: S) -> Option<Result<S::Ok, S::Error>> {
        // let mut serializer = &mut <dyn erased_serde::Serializer>::erase(serializer);
        let mut serializer = Some(serializer);

        poll_fn(|cx| match self.src.as_mut().poll_next_value2(cx) {
            Poll::Ready(Some(())) => {
                let value = self.src.take();
                let result = erased_serde::serialize(value, serializer.take().unwrap());
                Poll::Ready(Some(result))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        })
        .await
    }

    // TODO: Should this implement `Stream`. Well it can't cause serializer.
}

impl fmt::Debug for ProcedureStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}

trait A {
    fn poll_next_value(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        serializer: &mut dyn erased_serde::Serializer,
    ) -> Poll<Option<()>>;

    fn poll_next_value2<'a>(self: Pin<&'a mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>>;

    // TODO: Merge this return type of `Self::poll_next_value2`.
    fn take(&self) -> &dyn erased_serde::Serialize;
}

pin_project! {
    struct StreamToA<S: Stream>{
        #[pin]
        src: S,
        value: Option<S::Item>,
    }
}

impl<S: Stream> A for StreamToA<S>
where
    S::Item: Serialize, // TODO: Drop this bound!!!
{
    fn poll_next_value(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        serializer: &mut dyn erased_serde::Serializer,
    ) -> Poll<Option<()>> {
        let this = self.project();
        match this.src.poll_next(cx) {
            Poll::Ready(Some(value)) => {
                value.erased_serialize(serializer).unwrap();
                Poll::Ready(Some(()))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    // TODO: Cleanup this impl's pattern matching.
    fn poll_next_value2<'a>(mut self: Pin<&'a mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        let this = self.as_mut().project();
        match this.src.poll_next(cx) {
            Poll::Ready(Some(value)) => {
                *this.value = Some(value);
                Poll::Ready(Some(()))
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => return Poll::Pending,
        }
    }

    fn take(&self) -> &dyn erased_serde::Serialize {
        self.value.as_ref().unwrap()
    }
}
