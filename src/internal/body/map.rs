use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use pin_project_lite::pin_project;

pin_project! {
    /// A function for mapping a body into a future.
    #[must_use = "streams do nothing unless polled"]
    pub(crate) struct Map<Fut: Future, Result> {
        #[pin]
        future: Option<Fut>,
        func: fn(Fut::Output) -> Result,
    }
}

impl<Fut: Future, Result> Map<Fut, Result> {
    pub fn new(future: Fut, func: fn(Fut::Output) -> Result) -> Self {
        Self {
            future: Some(future),
            func,
        }
    }
}

impl<Fut: Future, Result> Future for Map<Fut, Result> {
    type Output = Result;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let v = match this.future.as_mut().as_pin_mut() {
            Some(fut) => ready!(fut.poll(cx)),
            None => panic!("`Map` polled after completion"),
        };

        this.future.set(None);
        Poll::Ready((this.func)(v))
    }
}
