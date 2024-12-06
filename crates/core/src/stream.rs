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
        T: Serialize + Send + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueFutureCompat {
                // TODO: Should we do this in a more efficient way???
                src: std::future::ready(value),
                value: None,
                done: false,
            }),
        }
    }

    /// TODO
    pub fn from_future<T, S>(src: S) -> Self
    where
        S: Future<Output = Result<T, ResolverError>> + Send + 'static,
        T: Serialize + Send + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueFutureCompat {
                src,
                value: None,
                done: false,
            }),
        }
    }

    /// TODO
    pub fn from_stream<T, S>(src: S) -> Self
    where
        S: Stream<Item = Result<T, ResolverError>> + Send + 'static,
        T: Serialize + Send + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueStreamCompat { src, value: None }),
        }
    }

    // TODO: I'm not sure if we should keep this or not?
    // The crate `futures`'s flatten stuff doesn't handle it how we need it so maybe we could patch that instead of having this special case???
    // This is a special case because we need to ensure the `size_hint` is correct.
    /// TODO
    pub fn from_future_stream<T, F, S>(src: F) -> Self
    where
        F: Future<Output = Result<S, ResolverError>> + Send + 'static,
        S: Stream<Item = Result<T, ResolverError>> + Send + 'static,
        T: Serialize + Send + 'static, // TODO: Drop `Serialize`!!!
    {
        Self {
            src: Box::pin(DynReturnValueStreamFutureCompat::Future { src }),
        }
    }

    // TODO: Rename and replace `Self::from_future_stream`???
    // TODO: I'm not sure if we should keep this or not?
    // The crate `futures`'s flatten stuff doesn't handle it how we need it so maybe we could patch that instead of having this special case???
    // This is a special case because we need to ensure the `size_hint` is correct.
    /// TODO
    pub fn from_future_procedure_stream<F>(src: F) -> Self
    where
        F: Future<Output = Result<Self, ResolverError>> + Send + 'static,
    {
        Self {
            src: Box::pin(DynReturnValueFutureProcedureStreamCompat::Future { src }),
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
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        self.src.size_hint()
    }

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

trait DynReturnValue: Send {
    fn poll_next_value<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>>;

    fn value(&self) -> &dyn erased_serde::Serialize;

    fn size_hint(&self) -> (usize, Option<usize>);
}

pin_project! {
    struct DynReturnValueFutureCompat<T, S>{
        #[pin]
        src: S,
        value: Option<T>,
        done: bool,
    }
}

impl<T: Send, S: Future<Output = Result<T, ResolverError>> + Send> DynReturnValue
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
            Poll::Ready(value) => {
                *this.done = true;
                Poll::Ready(Some(match value {
                    Ok(value) => {
                        *this.value = Some(value);

                        Ok(())
                    }
                    Err(err) => Err(err),
                }))
            }
            Poll::Pending => return Poll::Pending,
        }
    }

    fn value(&self) -> &dyn erased_serde::Serialize {
        self.value
            .as_ref()
            // Attempted to access value when `Poll::Ready(None)` was not returned.
            .expect("unreachable")
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }
        (1, Some(1))
    }
}

pin_project! {
    struct DynReturnValueStreamCompat<T, S>{
        #[pin]
        src: S,
        value: Option<T>,
    }
}

impl<T: Send, S: Stream<Item = Result<T, ResolverError>> + Send> DynReturnValue
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.src.size_hint()
    }
}

pin_project! {
    #[project = DynReturnValueStreamFutureCompatProj]
    enum DynReturnValueStreamFutureCompat<T, F, S> {
        Future {
            #[pin] src: F,
        },
        Stream {
            #[pin] src: S,
            value: Option<T>,
        }
    }
}

impl<T, F, S> DynReturnValue for DynReturnValueStreamFutureCompat<T, F, S>
where
    T: Serialize + Send, // TODO: Drop `Serialize` bound!!!
    F: Future<Output = Result<S, ResolverError>> + Send + 'static,
    S: Stream<Item = Result<T, ResolverError>> + Send,
{
    // TODO: Cleanup this impl's pattern matching.
    fn poll_next_value<'a>(
        mut self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>> {
        loop {
            return match self.as_mut().project() {
                DynReturnValueStreamFutureCompatProj::Future { src } => match src.poll(cx) {
                    Poll::Ready(Ok(result)) => {
                        self.as_mut().set(DynReturnValueStreamFutureCompat::Stream {
                            src: result,
                            value: None,
                        });
                        continue;
                    }
                    Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(err))),
                    Poll::Pending => return Poll::Pending,
                },
                DynReturnValueStreamFutureCompatProj::Stream { src, value } => {
                    let _ = value.take(); // Reset value to ensure `take` being misused causes it to panic.
                    match src.poll_next(cx) {
                        Poll::Ready(Some(v)) => Poll::Ready(Some(match v {
                            Ok(v) => {
                                *value = Some(v);
                                Ok(())
                            }
                            Err(err) => Err(err),
                        })),
                        Poll::Ready(None) => Poll::Ready(None),
                        Poll::Pending => Poll::Pending,
                    }
                }
            };
        }
    }

    fn value(&self) -> &dyn erased_serde::Serialize {
        match self {
            // Attempted to acces value before first `Poll::Ready` was returned.
            Self::Future { .. } => panic!("unreachable"),
            Self::Stream { value, .. } => value
                .as_ref()
                // Attempted to access value when `Poll::Ready(None)` was not returned.
                .expect("unreachable"),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Future { .. } => (0, None),
            Self::Stream { src, .. } => src.size_hint(),
        }
    }
}

pin_project! {
    #[project = DynReturnValueFutureProcedureStreamCompatProj]
    enum DynReturnValueFutureProcedureStreamCompat<F> {
        Future {
            #[pin] src: F,
        },
        Inner {
            src: ProcedureStream,
        }
    }
}

impl<F> DynReturnValue for DynReturnValueFutureProcedureStreamCompat<F>
where
    F: Future<Output = Result<ProcedureStream, ResolverError>> + Send + 'static,
{
    // TODO: Cleanup this impl's pattern matching.
    fn poll_next_value<'a>(
        mut self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ResolverError>>> {
        loop {
            return match self.as_mut().project() {
                DynReturnValueFutureProcedureStreamCompatProj::Future { src } => match src.poll(cx)
                {
                    Poll::Ready(Ok(result)) => {
                        self.as_mut()
                            .set(DynReturnValueFutureProcedureStreamCompat::Inner { src: result });
                        continue;
                    }
                    Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(err))),
                    Poll::Pending => return Poll::Pending,
                },
                DynReturnValueFutureProcedureStreamCompatProj::Inner { src } => {
                    src.src.as_mut().poll_next_value(cx)
                }
            };
        }
    }

    fn value(&self) -> &dyn erased_serde::Serialize {
        match self {
            // Attempted to acces value before first `Poll::Ready` was returned.
            Self::Future { .. } => panic!("unreachable"),
            Self::Inner { src } => src.src.value(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Future { .. } => (0, None),
            Self::Inner { src } => src.src.size_hint(),
        }
    }
}
