use std::pin::Pin;

use futures::Stream;

use crate::error::ExecError;

// It is expected that the type remains the same for all items of a single stream! It's ok for panic's if this is violated.
//
// TODO: Can this be `pub(crate)`??? -> Right now `Layer` is the problem
pub enum ValueOrBytes {
    Value(serde_json::Value),
    Bytes(Vec<u8>),
}

pub(crate) type StreamItem = Result<ValueOrBytes, ExecError>;
pub(crate) type ErasedBody = Pin<Box<dyn Stream<Item = StreamItem> + Send>>;

// pub(crate) type ErasedBody = BodyInternal<Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>>;

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

#[derive(Debug)] // TODO: Better debug impl
pub struct BytesBody {}
