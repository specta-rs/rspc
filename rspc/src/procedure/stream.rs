use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use super::output::ProcedureOutput;

enum Inner {
    Value(ProcedureOutput),
    Future(Pin<Box<dyn Future<Output = ProcedureOutput> + Send>>),
    Stream(Pin<Box<dyn Stream<Item = ProcedureOutput> + Send>>),
}

pub struct ProcedureStream(Option<Inner>);

impl ProcedureStream {
    pub fn from_value(value: ProcedureOutput) -> Self {
        Self(Some(Inner::Value(value)))
    }

    pub fn from_future<F>(future: F) -> Self
    where
        F: Future<Output = ProcedureOutput> + Send + 'static,
    {
        Self(Some(Inner::Future(Box::pin(future))))
    }

    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = ProcedureOutput> + Send + 'static,
    {
        Self(Some(Inner::Stream(Box::pin(stream))))
    }
}

impl Stream for ProcedureStream {
    type Item = ProcedureOutput;

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
