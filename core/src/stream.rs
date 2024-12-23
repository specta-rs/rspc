use core::fmt;
use std::{
    future::poll_fn,
    panic::{catch_unwind, AssertUnwindSafe},
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use pin_project_lite::pin_project;
use serde::Serialize;

use crate::ProcedureError;

/// TODO
#[must_use = "ProcedureStream does nothing unless polled"]
pub struct ProcedureStream(Result<Pin<Box<dyn DynReturnValue>>, Option<ProcedureError>>);

impl ProcedureStream {
    /// TODO
    pub fn from_stream<T, S>(s: S) -> Self
    where
        S: Stream<Item = Result<T, ProcedureError>> + Send + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        Self(Ok(Box::pin(DynReturnImpl {
            src: s,
            unwound: false,
            value: None,
        })))
    }

    /// TODO
    pub fn from_stream_value<T, S>(s: S) -> Self
    where
        S: Stream<Item = Result<T, ProcedureError>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        Self(todo!())
    }

    /// TODO
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            Ok(v) => v.size_hint(),
            Err(_) => (1, Some(1)),
        }
    }

    /// TODO
    pub fn poll_next(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<impl Serialize + Send + Sync + '_, ProcedureError>>> {
        match &mut self.0 {
            Ok(v) => v
                .as_mut()
                .poll_next_value(cx)
                .map(|v| v.map(|v| v.map(|_: ()| self.0.as_mut().expect("checked above").value()))),
            Err(err) => Poll::Ready(err.take().map(Err)),
        }
    }

    /// TODO
    pub async fn next(
        &mut self,
    ) -> Option<Result<impl Serialize + Send + Sync + '_, ProcedureError>> {
        match self {
            Self(Ok(v)) => poll_fn(|cx| v.as_mut().poll_next_value(cx))
                .await
                .map(|v| v.map(|_: ()| self.0.as_mut().expect("checked above").value())),
            Self(Err(err)) => err.take().map(Err),
        }
    }

    /// TODO
    pub async fn next_status(&mut self) -> (u16, bool) {
        // TODO: Panic if it isn't the start of the stream or not???

        // TODO: Poll till the first return value and return it's code.

        // TODO: Should we keep polling so we can tell if it's a value or a stream for the content type???

        // todo!();
        (200, false)
    }

    /// TODO
    // TODO: Should error be `String` type?
    pub fn map<F: FnMut(ProcedureStreamValue) -> Result<T, String> + Unpin, T>(
        self,
        map: F,
    ) -> ProcedureStreamMap<F, T> {
        ProcedureStreamMap { stream: self, map }
    }
}

pub struct ProcedureStreamMap<F: FnMut(ProcedureStreamValue) -> Result<T, String> + Unpin, T> {
    stream: ProcedureStream,
    map: F,
}

impl<F: FnMut(ProcedureStreamValue) -> Result<T, String> + Unpin, T> Stream
    for ProcedureStreamMap<F, T>
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        match this.stream.0.as_mut() {
            Ok(v) => v.as_mut().poll_next_value(cx).map(|v| {
                v.map(|v| match v {
                    Ok(()) => match (this.map)(ProcedureStreamValue(
                        this.stream.0.as_mut().expect("checked above").value(),
                    )) {
                        Ok(v) => v,
                        // TODO: Exposing this error to the client or not?
                        // TODO: Error type???
                        Err(err) => todo!(),
                    },
                    Err(err) => todo!("{err:?}"),
                })
            }),
            Err(err) => todo!(),
        }
    }
}

// TODO: name
pub struct ProcedureStreamValue<'a>(&'a (dyn erased_serde::Serialize + Send + Sync));
// TODO: `Debug`, etc traits

impl<'a> Serialize for ProcedureStreamValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl From<ProcedureError> for ProcedureStream {
    fn from(err: ProcedureError) -> Self {
        Self(Err(Some(err)))
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
    ) -> Poll<Option<Result<(), ProcedureError>>>;

    fn value(&self) -> &(dyn erased_serde::Serialize + Send + Sync);

    fn size_hint(&self) -> (usize, Option<usize>);
}

pin_project! {
    struct DynReturnImpl<T, S>{
        #[pin]
        src: S,
        unwound: bool,
        value: Option<T>,
    }
}

impl<T, S: Stream<Item = Result<T, ProcedureError>> + Send + 'static> DynReturnValue
    for DynReturnImpl<T, S>
where
    T: Send + Sync + Serialize,
{
    fn poll_next_value<'a>(
        mut self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), ProcedureError>>> {
        if self.unwound {
            // The stream is now done.
            return Poll::Ready(None);
        }

        let this = self.as_mut().project();
        let r = catch_unwind(AssertUnwindSafe(|| {
            let _ = this.value.take(); // Reset value to ensure `take` being misused causes it to panic.
            this.src.poll_next(cx).map(|v| {
                v.map(|v| {
                    v.map(|v| {
                        *this.value = Some(v);
                        ()
                    })
                })
            })
        }));

        match r {
            Ok(v) => v,
            Err(err) => {
                *this.unwound = true;
                Poll::Ready(Some(Err(ProcedureError::Unwind(err))))
            }
        }
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
