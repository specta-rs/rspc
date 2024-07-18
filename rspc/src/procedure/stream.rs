use std::{
    error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, TryFutureExt, TryStreamExt};

use crate::Error;

use super::{output::ProcedureOutput, ResolverError};

enum Inner {
    Value(Result<ProcedureOutput, ResolverError>),
    Future(Pin<Box<dyn Future<Output = Result<ProcedureOutput, ResolverError>> + Send>>),
    Stream(Pin<Box<dyn Stream<Item = Result<ProcedureOutput, ResolverError>> + Send>>),
}

pub struct ProcedureStream(Option<Inner>);

impl ProcedureStream {
    pub fn from_value<TError>(value: Result<ProcedureOutput, TError>) -> Self
    where
        TError: Error,
    {
        Self(Some(Inner::Value(value.map_err(ResolverError::new))))
    }

    pub fn from_future<F, TError>(future: F) -> Self
    where
        F: Future<Output = Result<ProcedureOutput, TError>> + Send + 'static,
        TError: Error,
    {
        Self(Some(Inner::Future(Box::pin(
            future.map_err(ResolverError::new),
        ))))
    }

    pub fn from_stream<S, TError>(stream: S) -> Self
    where
        S: Stream<Item = Result<ProcedureOutput, TError>> + Send + 'static,
        TError: Error,
    {
        Self(Some(Inner::Stream(Box::pin(
            stream.map_err(ResolverError::new),
        ))))
    }
}

impl Stream for ProcedureStream {
    type Item = Result<ProcedureOutput, ResolverError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.as_mut() {
            Some(Inner::Value(_)) => {
                let Inner::Value(value) = self.0.take().expect("checked above") else {
                    panic!("checked above");
                };
                Poll::Ready(Some(value))
            }
            Some(Inner::Future(future)) => future.as_mut().poll(cx).map(|v| {
                self.0 = None;
                Some(v)
            }),
            Some(Inner::Stream(stream)) => stream.as_mut().poll_next(cx),
            None => Poll::Ready(None),
        }
    }
}
