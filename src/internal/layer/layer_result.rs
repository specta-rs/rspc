use std::{
    future::Future,
    marker::PhantomData,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

enum Inner<'a, F>
where
    F: Future + 'a,
    F::Output: Stream + Send + 'static,
{
    Future(F),
    Stream(F::Output),
    Phantom(PhantomData<&'a ()>),
}

impl<'a, F> DynLayerResultImpl<'a, <F::Output as Stream>::Item> for Inner<'a, F>
where
    F: Future + Send + 'a,
    F::Output: Stream + Send + 'static,
{
    fn todo(
        self: Box<Self>,
    ) -> Box<dyn Stream<Item = <F::Output as Stream>::Item> + Send + 'static> {
        match &*self {
            Self::Future(_) => unreachable!(),
            Self::Stream(_) => {
                let result: Box<dyn Stream<Item = <F::Output as Stream>::Item> + Send + 'a> = self;
                // TODO: Explain why this is safe
                let result: Box<dyn Stream<Item = <F::Output as Stream>::Item> + Send + 'static> =
                    unsafe { std::mem::transmute(result) };
                result
            }
            Self::Phantom(_) => unreachable!(),
        }
    }
}

impl<'a, F> Future for Inner<'a, F>
where
    F: Future + Send + 'a,
    F::Output: Stream + Send + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

impl<'a, F> Stream for Inner<'a, F>
where
    F: Future + Send + 'a,
    F::Output: Stream + Send + 'static,
{
    type Item = <F::Output as Stream>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

// It would be pog if the `Future` you could return directly instead of requiring `take` but it would be a cycle.
trait DynLayerResultImpl<'a, T>: Future<Output = ()> {
    // This method *will* panic if the `Future` hasn't been awaited.
    fn todo(self: Box<Self>) -> Box<dyn Stream<Item = T> + Send>;
}

pub(crate) struct DynLayerResult<'a, T> {
    inner: Option<Box<dyn DynLayerResultImpl<'a, T> + Send + 'a>>,
}

impl<'a, T: Send> DynLayerResult<'a, T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Future + Send + 'a,
        F::Output: Stream<Item = T> + Send + 'static,
    {
        Self {
            inner: Some(Box::new(Inner::Future(f))),
        }
    }
}

impl<'a, T> Future for DynLayerResult<'a, T> {
    type Output = Pin<Box<dyn Stream<Item = T> + Send>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Explain safety
        let inner = unsafe {
            self.as_mut()
                .map_unchecked_mut(|s| &mut **s.inner.as_mut().unwrap())
        };

        match inner.poll(cx) {
            Poll::Ready(()) => {
                let inner = mem::replace(&mut self.inner, None).unwrap();
                // TODO: Can we avoid this `unsafe`???
                Poll::Ready(unsafe { Pin::new_unchecked(inner.todo()) })
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
