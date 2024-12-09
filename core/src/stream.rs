use core::fmt;
use std::{
    future::poll_fn,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use pin_project_lite::pin_project;
use serde::Serialize;

use crate::ResolverError;

/// TODO
#[must_use = "ProcedureStream does nothing unless polled"]
pub struct ProcedureStream(Pin<Box<dyn DynReturnValue>>);

impl ProcedureStream {
    /// TODO
    pub fn from_stream<T, S>(s: S) -> Self
    where
        S: Stream<Item = Result<T, ResolverError>> + Send + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        Self(Box::pin(DynReturnImpl {
            src: s,
            value: None,
        }))
    }

    /// TODO
    pub fn from_stream_value<T, S>(s: S) -> Self
    where
        S: Stream<Item = Result<T, ResolverError>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        Self(todo!())
    }

    /// TODO
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    /// TODO
    pub fn poll_next(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<impl Serialize + Send + Sync + '_, ResolverError>>> {
        self.0
            .as_mut()
            .poll_next_value(cx)
            .map(|v| v.map(|v| v.map(|_: ()| self.0.value())))
    }

    /// TODO
    pub async fn next(
        &mut self,
    ) -> Option<Result<impl Serialize + Send + Sync + '_, ResolverError>> {
        poll_fn(|cx| self.0.as_mut().poll_next_value(cx))
            .await
            .map(|v| v.map(|_: ()| self.0.value()))
    }
}

impl fmt::Debug for ProcedureStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}

trait DynReturnValue: Send {
    fn poll_next_value<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>>;

    fn value(&self) -> &(dyn erased_serde::Serialize + Send + Sync);

    fn size_hint(&self) -> (usize, Option<usize>);
}

pin_project! {
    struct DynReturnImpl<T, S>{
        #[pin]
        src: S,
        value: Option<T>,
    }
}

impl<T, S: Stream<Item = Result<T, ResolverError>> + Send + 'static> DynReturnValue
    for DynReturnImpl<T, S>
where
    T: Send + Sync + Serialize,
{
    fn poll_next_value<'a>(
        mut self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>> {
        let this = self.as_mut().project();
        let _ = this.value.take(); // Reset value to ensure `take` being misused causes it to panic.
        this.src.poll_next(cx).map(|v| {
            v.map(|v| {
                v.map(|v| {
                    *this.value = Some(v);
                    ()
                })
            })
        })
    }

    fn value(&self) -> &(dyn erased_serde::Serialize + Send + Sync) {
        self.value
            .as_ref()
            // Attempted to access value when `Poll::Ready(None)` was not returned.
            .expect("unreachable")
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.src.size_hint()
    }
}
