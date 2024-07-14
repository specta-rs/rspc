use std::{
    error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, TryFutureExt, TryStreamExt};

use super::output::ProcedureOutput;

type BoxError = Box<dyn std::error::Error + Send + 'static>;
fn box_error<T>(err: T) -> BoxError
where
    T: error::Error + Send + 'static,
{
    Box::new(err)
}

enum Inner {
    Value(Result<ProcedureOutput, BoxError>),
    Future(Pin<Box<dyn Future<Output = Result<ProcedureOutput, BoxError>> + Send>>),
    Stream(Pin<Box<dyn Stream<Item = Result<ProcedureOutput, BoxError>> + Send>>),
}

pub struct ProcedureStream(Option<Inner>);

impl ProcedureStream {
    pub fn from_value<TError>(value: Result<ProcedureOutput, TError>) -> Self
    where
        TError: error::Error + Send + 'static,
    {
        Self(Some(Inner::Value(value.map_err(box_error))))
    }

    pub fn from_future<F, TError>(future: F) -> Self
    where
        F: Future<Output = Result<ProcedureOutput, TError>> + Send + 'static,
        TError: error::Error + Send + 'static,
    {
        Self(Some(Inner::Future(Box::pin(future.map_err(box_error)))))
    }

    pub fn from_stream<S, TError>(stream: S) -> Self
    where
        S: Stream<Item = Result<ProcedureOutput, TError>> + Send + 'static,
        TError: error::Error + Send + 'static,
    {
        Self(Some(Inner::Stream(Box::pin(stream.map_err(box_error)))))
    }
}

impl Stream for ProcedureStream {
    type Item = Result<ProcedureOutput, BoxError>;

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
