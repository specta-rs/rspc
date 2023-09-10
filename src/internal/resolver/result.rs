//! TODO
//!
//! TODO: Rename this file

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{internal::Body, ExecError};

use futures::Stream;
use pin_project_lite::pin_project;
use serde_json::Value;

#[cfg(feature = "tracing")]
type Inner = tracing::Span;

#[cfg(not(feature = "tracing"))]
type Inner = ();

pin_project! {
    pub struct StreamToBody<S> {
        #[pin]
        stream: S,
        span: Option<Inner>
    }
}

impl<S: Stream<Item = Result<Value, ExecError>> + Send + 'static> Body for StreamToBody<S> {
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Value, ExecError>>> {
        let this = self.project();

        #[cfg(feature = "tracing")]
        let _span = this.span.as_ref().map(|s| s.enter());

        this.stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
