use core::fmt;
use std::{
    future::{poll_fn, Future},
    pin::Pin,
    task::{Context, Poll},
};

use erased_serde::Serialize;
use futures_core::Stream;
use pin_project_lite::pin_project;
use serde::Serializer;

use crate::{ProcedureError, ResolverError};

/// TODO
// TODO: Rename this type.
pub struct ProcedureStream {
    src: Pin<Box<dyn DynReturnValue>>,
}

impl ProcedureStream {
    /// TODO
    pub fn from_value<T>(value: Result<T, ResolverError>) -> Self
    where
        T: Serialize + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueFutureCompat {
                // TODO: Should we do this in a more efficient way???
                src: std::future::ready(value),
                value: None,
            }),
        }
    }

    /// TODO
    pub fn from_future<T, S>(src: S) -> Self
    where
        S: Future<Output = Result<T, ResolverError>> + 'static,
        T: Serialize + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueFutureCompat { src, value: None }),
        }
    }

    /// TODO
    pub fn from_stream<T, S>(src: S) -> Self
    where
        S: Stream<Item = Result<T, ResolverError>> + 'static,
        T: Serialize + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueStreamCompat { src, value: None }),
        }
    }

    // /// TODO
    // ///
    // /// TODO: This method doesn't allow reusing the serializer between polls. Maybe remove it???
    // pub fn poll_next<S: Serializer>(
    //     &mut self,
    //     cx: &mut Context<'_>,
    //     serializer: S,
    // ) -> Poll<Option<()>> {
    //     let mut serializer = &mut <dyn erased_serde::Serializer>::erase(serializer);

    //     self.src.as_mut().poll_next_value(cx)
    // }

    // TODO: Fn to get syncronous value???

    /// TODO
    pub async fn next<S: Serializer>(
        &mut self,
        serializer: S,
    ) -> Option<Result<S::Ok, ProcedureError<S>>> {
        let mut serializer = Some(serializer);

        poll_fn(|cx| match self.src.as_mut().poll_next_value(cx) {
            Poll::Ready(Some(result)) => Poll::Ready(Some(match result {
                Ok(()) => {
                    let value = self.src.value();
                    erased_serde::serialize(value, serializer.take().unwrap())
                        .map_err(ProcedureError::Serializer)
                }
                Err(err) => Err(err.into()),
            })),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        })
        .await
    }
}

impl fmt::Debug for ProcedureStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}

trait DynReturnValue {
    fn poll_next_value<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>>;

    fn value(&self) -> &dyn erased_serde::Serialize;
}

pin_project! {
    struct DynReturnValueFutureCompat<T, S>{
        #[pin]
        src: S,
        value: Option<T>,
    }
}

impl<T, S: Future<Output = Result<T, ResolverError>>> DynReturnValue
    for DynReturnValueFutureCompat<T, S>
where
    T: Serialize, // TODO: Drop this bound!!!
{
    // TODO: Cleanup this impl's pattern matching.
    fn poll_next_value<'a>(
        mut self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>> {
        let this = self.as_mut().project();
        let _ = this.value.take(); // Reset value to ensure `take` being misused causes it to panic.
        match this.src.poll(cx) {
            Poll::Ready(value) => Poll::Ready(Some(match value {
                Ok(value) => {
                    *this.value = Some(value);
                    Ok(())
                }
                Err(err) => Err(err),
            })),
            Poll::Pending => return Poll::Pending,
        }
    }

    fn value(&self) -> &dyn erased_serde::Serialize {
        self.value
            .as_ref()
            // Attempted to access value when `Poll::Ready(None)` was not returned.
            .expect("unreachable")
    }
}

pin_project! {
    struct DynReturnValueStreamCompat<T, S>{
        #[pin]
        src: S,
        value: Option<T>,
    }
}

impl<T, S: Stream<Item = Result<T, ResolverError>>> DynReturnValue
    for DynReturnValueStreamCompat<T, S>
where
    T: Serialize, // TODO: Drop this bound!!!
{
    // TODO: Cleanup this impl's pattern matching.
    fn poll_next_value<'a>(
        mut self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>> {
        let this = self.as_mut().project();
        let _ = this.value.take(); // Reset value to ensure `take` being misused causes it to panic.
        match this.src.poll_next(cx) {
            Poll::Ready(Some(value)) => Poll::Ready(Some(match value {
                Ok(value) => {
                    *this.value = Some(value);
                    Ok(())
                }
                Err(err) => Err(err),
            })),
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => return Poll::Pending,
        }
    }

    fn value(&self) -> &dyn erased_serde::Serialize {
        self.value
            .as_ref()
            // Attempted to access value when `Poll::Ready(None)` was not returned.
            .expect("unreachable")
    }
}
