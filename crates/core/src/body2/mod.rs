use std::pin::Pin;

use futures::Stream;
use serde_json::Value;

use crate::error::ExecError;

pub mod cursed;

// It is expected that the type remains the same for all items of a single stream! It's ok for panic's if this is violated.
//
// TODO: Can this be `pub(crate)`??? -> Right now `Layer` is the problem
#[derive(Debug)]
pub enum ValueOrBytes {
    Value(serde_json::Value),
    Bytes(Vec<u8>),
}

pub(crate) type StreamItem = Result<ValueOrBytes, ExecError>;
pub(crate) type ErasedBody = Pin<Box<dyn Stream<Item = StreamItem> + Send>>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Body {
    // Derived when `Stream` has one and only one item
    Value(serde_json::Value),
    // Derived from `ValueOrBytes`
    Stream(StreamBody),
    // Derived from `ValueOrBytes`
    Bytes(BytesBody), // TODO: Implement this
}

#[derive(Debug)] // TODO: Better debug impl
pub struct StreamBody {}

impl Stream for StreamBody {
    type Item = Result<Value, ExecError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // Set into thread, hey I want the next value

        // Suspense
        todo!();
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[derive(Debug)] // TODO: Better debug impl
pub struct BytesBody {}
