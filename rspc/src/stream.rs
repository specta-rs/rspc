use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::StreamExt;

/// Return a [`Stream`](futures::Stream) of values from a [`Procedure::query`](procedure::ProcedureBuilder::query) or [`Procedure::mutation`](procedure::ProcedureBuilder::mutation).
///
/// ## Why not a subscription?
///
/// A [`subscription`](procedure::ProcedureBuilder::subscription) must return a [`Stream`](futures::Stream) so it would be fair to question when you would use this.
///
/// A [`query`](procedure::ProcedureBuilder::query) or [`mutation`](procedure::ProcedureBuilder::mutation) produce a single result where a subscription produces many discrete values.
///
/// Using [`rspc::Stream`](Self) within a query or mutation will result in your procedure returning a collection (Eg. `Vec`) of [`Stream::Item`](futures::Stream) on the frontend.
///
/// This means it would be well suited for streaming the result of a computation or database query while a subscription would be well suited for a chat room.
///
/// ## Usage
/// **WARNING**: This example shows the low-level procedure API. You should refer to [`Rspc`](crate::Rspc) for the high-level API.
/// ```rust
/// use futures::stream::once;
///
/// <Procedure>::builder().query(|_, _: ()| async move { rspc::Stream(once(async move { 42 })) });
/// ```
///
pub struct Stream<S: futures_util::Stream>(pub S);

// WARNING: We can not add an implementation for `Debug` without breaking `rspc_tracing`

impl<S: futures_util::Stream + Default> Default for Stream<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S: futures_util::Stream + Clone> Clone for Stream<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: futures_util::Stream> futures_util::Stream for Stream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // TODO: Using `pin-project-lite` would be nice but I don't think it supports tuple variants and I don't want the macros of `pin-project`.
        unsafe { self.map_unchecked_mut(|v| &mut v.0) }.poll_next_unpin(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
