use core::fmt;
use std::{
    cell::RefCell,
    future::{poll_fn, Future},
    panic::{catch_unwind, AssertUnwindSafe},
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll, Waker},
};

use futures_core::Stream;
use pin_project_lite::pin_project;
use serde::Serialize;

use crate::{DynOutput, ProcedureError};

thread_local! {
    static CAN_FLUSH: RefCell<bool> = RefCell::default();
    static SHOULD_FLUSH: RefCell<Option<bool>> = RefCell::default();
}

/// TODO
pub async fn flush() {
    if CAN_FLUSH.with(|v| *v.borrow()) {
        let mut pending = true;
        poll_fn(|_| {
            if pending {
                pending = false;
                SHOULD_FLUSH.replace(Some(true));
                return Poll::Pending;
            }

            Poll::Ready(())
        })
        .await;
    }
}

enum Inner {
    Dyn(Pin<Box<dyn DynReturnValue>>),
    Value(Option<ProcedureError>),
}

/// TODO
#[must_use = "`ProcedureStream` does nothing unless polled"]
pub struct ProcedureStream {
    inner: Inner,
    // If `None` flushing is allowed.
    // This is the default but will also be set after `flush` is called.
    //
    // If `Some` then `flush` must be called before the next value is yielded.
    // Will poll until the first value and then return `Poll::Pending` and record the waker.
    // The stored value will be yielded immediately after `flush` is called.
    flush: Option<Waker>,
    // This is set `true` if `Poll::Ready` is called while `flush` is `Some`.
    // This informs the stream to yield the value immediately when `flush` is `None` again.
    pending_value: bool, // TODO: Could we just check for a value on `inner`? Less chance of panic in the case of a bug.
}

impl From<ProcedureError> for ProcedureStream {
    fn from(err: ProcedureError) -> Self {
        Self {
            inner: Inner::Value(Some(err)),
            flush: None,
            pending_value: false,
        }
    }
}

impl ProcedureStream {
    /// TODO
    pub fn from_stream<T, S>(s: S) -> Self
    where
        S: Stream<Item = Result<T, ProcedureError>> + Send + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        Self {
            inner: Inner::Dyn(Box::pin(GenericDynReturnValue {
                inner: s,
                poll: |s, cx| s.poll_next(cx),
                size_hint: |s| s.size_hint(),
                resolved: |_| true,
                as_value: |v| {
                    DynOutput::new_serialize(
                        v.as_mut()
                            // Error's are caught before `as_value` is called.
                            .expect("unreachable")
                            .as_mut()
                            // Attempted to access value when `Poll::Ready(None)` was not returned.
                            .expect("unreachable"),
                    )
                },
                flushed: false,
                unwound: false,
                value: None,
            })),
            flush: None,
            pending_value: false,
        }
    }

    /// TODO
    pub fn from_future<T, F>(f: F) -> Self
    where
        F: Future<Output = Result<T, ProcedureError>> + Send + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        pin_project! {
            #[project = ReprProj]
            struct Repr<F> {
                #[pin]
                inner: Option<F>,
            }
        }

        Self {
            inner: Inner::Dyn(Box::pin(GenericDynReturnValue {
                inner: Repr { inner: Some(f) },
                poll: |f, cx| {
                    let mut this = f.project();
                    let v = match this.inner.as_mut().as_pin_mut() {
                        Some(fut) => ready!(fut.poll(cx)),
                        None => return Poll::Ready(None),
                    };

                    this.inner.set(None);
                    Poll::Ready(Some(v))
                },
                size_hint: |f| {
                    if f.inner.is_some() {
                        (1, Some(1))
                    } else {
                        (0, Some(0))
                    }
                },
                as_value: |v| {
                    DynOutput::new_serialize(
                        v.as_mut()
                            // Error's are caught before `as_value` is called.
                            .expect("unreachable")
                            .as_mut()
                            // Attempted to access value when `Poll::Ready(None)` was not returned.
                            .expect("unreachable"),
                    )
                },
                resolved: |f| f.inner.is_none(),
                flushed: false,
                unwound: false,
                value: None,
            })),
            flush: None,
            pending_value: false,
        }
    }

    /// TODO
    pub fn from_future_stream<T, F, S>(f: F) -> Self
    where
        F: Future<Output = Result<S, ProcedureError>> + Send + 'static,
        S: Stream<Item = Result<T, ProcedureError>> + Send + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        pin_project! {
            #[project = ReprProj]
            enum Repr<F, S> {
                Future {
                    #[pin]
                    inner: F,
                },
                Stream {
                    #[pin]
                    inner: S,
                },
            }
        }

        Self {
            inner: Inner::Dyn(Box::pin(GenericDynReturnValue {
                inner: Repr::<F, S>::Future { inner: f },
                poll: |mut f, cx| loop {
                    let this = f.as_mut().project();
                    match this {
                        ReprProj::Future { inner } => {
                            let Poll::Ready(Ok(stream)) = inner.poll(cx) else {
                                return Poll::Pending;
                            };

                            f.set(Repr::Stream { inner: stream });
                            continue;
                        }
                        ReprProj::Stream { inner } => return inner.poll_next(cx),
                    }
                },
                size_hint: |_| (1, Some(1)),
                resolved: |f| matches!(f, Repr::Stream { .. }),
                as_value: |v| {
                    DynOutput::new_serialize(
                        v.as_mut()
                            // Error's are caught before `as_value` is called.
                            .expect("unreachable")
                            .as_mut()
                            // Attempted to access value when `Poll::Ready(None)` was not returned.
                            .expect("unreachable"),
                    )
                },
                flushed: false,
                unwound: false,
                value: None,
            })),
            flush: None,
            pending_value: false,
        }
    }

    /// TODO
    pub fn from_stream_value<T, S>(s: S) -> Self
    where
        S: Stream<Item = Result<T, ProcedureError>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        Self {
            inner: Inner::Dyn(Box::pin(GenericDynReturnValue {
                inner: s,
                poll: |s, cx| s.poll_next(cx),
                size_hint: |s| s.size_hint(),
                resolved: |_| true,
                // We passthrough the whole `Option` intentionally.
                as_value: |v| DynOutput::new_value(v),
                flushed: false,
                unwound: false,
                value: None,
            })),
            flush: None,
            pending_value: false,
        }
    }

    /// TODO
    pub fn from_future_value<T, F>(f: F) -> Self
    where
        F: Future<Output = Result<T, ProcedureError>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        pin_project! {
            #[project = ReprProj]
            struct Repr<F> {
                #[pin]
                inner: Option<F>,
            }
        }

        Self {
            inner: Inner::Dyn(Box::pin(GenericDynReturnValue {
                inner: Repr { inner: Some(f) },
                poll: |f, cx| {
                    let mut this = f.project();
                    let v = match this.inner.as_mut().as_pin_mut() {
                        Some(fut) => ready!(fut.poll(cx)),
                        None => return Poll::Ready(None),
                    };

                    this.inner.set(None);
                    Poll::Ready(Some(v))
                },
                size_hint: |f| {
                    if f.inner.is_some() {
                        (1, Some(1))
                    } else {
                        (0, Some(0))
                    }
                },
                as_value: |v| DynOutput::new_value(v),
                resolved: |f| f.inner.is_none(),
                flushed: false,
                unwound: false,
                value: None,
            })),
            flush: None,
            pending_value: false,
        }
    }

    /// TODO
    pub fn from_future_stream_value<T, F, S>(f: F) -> Self
    where
        F: Future<Output = Result<S, ProcedureError>> + Send + 'static,
        S: Stream<Item = Result<T, ProcedureError>> + Send + 'static,
        T: Send + Sync + 'static,
    {
        pin_project! {
            #[project = ReprProj]
            enum Repr<F, S> {
                Future {
                    #[pin]
                    inner: F,
                },
                Stream {
                    #[pin]
                    inner: S,
                },
            }
        }

        Self {
            inner: Inner::Dyn(Box::pin(GenericDynReturnValue {
                inner: Repr::<F, S>::Future { inner: f },
                poll: |mut f, cx| loop {
                    let this = f.as_mut().project();
                    match this {
                        ReprProj::Future { inner } => {
                            let Poll::Ready(Ok(stream)) = inner.poll(cx) else {
                                return Poll::Pending;
                            };

                            f.set(Repr::Stream { inner: stream });
                            continue;
                        }
                        ReprProj::Stream { inner } => return inner.poll_next(cx),
                    }
                },
                size_hint: |_| (1, Some(1)),
                resolved: |f| matches!(f, Repr::Stream { .. }),
                as_value: |v| DynOutput::new_value(v),
                flushed: false,
                unwound: false,
                value: None,
            })),
            flush: None,
            pending_value: false,
        }
    }

    /// By setting this the stream will delay returning any data until instructed by the caller (via `Self::stream`).
    ///
    /// This allows you to progress an entire runtime of streams until all of them are in a state ready to start returning responses.
    /// This mechanism allows anything that could need to modify the HTTP response headers to do so before the body starts being streamed.
    ///
    /// # Behaviour
    ///
    /// `ProcedureStream` will poll the underlying stream until the first value is ready.
    /// It will then return `Poll::Pending` and go inactive until `Self::stream` is called.
    /// When polled for the first time after `Self::stream` is called if a value was already ready it will be immediately returned.
    /// It is *guaranteed* that the stream will never yield `Poll::Ready` until `flush` is called if this is set.
    ///
    /// # Usage
    ///
    /// It's generally expected you will continue to poll the runtime until some criteria based on `Self::resolved` & `Self::flushable` is met on all streams.
    /// Once this is met you can call `Self::stream` on all of the streams at once to begin streaming data.
    ///
    pub fn require_manual_stream(mut self) -> Self {
        // TODO: When stablised replace with - https://doc.rust-lang.org/stable/std/task/struct.Waker.html#method.noop
        struct NoOpWaker;
        impl std::task::Wake for NoOpWaker {
            fn wake(self: std::sync::Arc<Self>) {}
        }

        // This `Arc` is inefficient but `Waker::noop` is coming soon which will solve it.
        self.flush = Some(Arc::new(NoOpWaker).into());
        self
    }

    /// Start streaming data.
    /// Refer to `Self::require_manual_stream` for more information.
    pub fn stream(&mut self) {
        if let Some(waker) = self.flush.take() {
            waker.wake();
        }
    }

    /// Will return `true` if the future has resolved.
    ///
    /// For a stream created via `Self::from_future*` this will be `true` once the future has resolved and for all other streams this will always be `true`.
    pub fn resolved(&self) -> bool {
        match &self.inner {
            Inner::Dyn(stream) => stream.resolved(),
            Inner::Value(_) => true,
        }
    }

    /// Will return `true` if the stream is ready to start streaming data.
    ///
    /// This is `false` until the `flush` function is called by the user.
    pub fn flushable(&self) -> bool {
        match &self.inner {
            Inner::Dyn(stream) => stream.flushed(),
            Inner::Value(_) => false,
        }
    }

    /// TODO
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            Inner::Dyn(stream) => stream.size_hint(),
            Inner::Value(_) => (1, Some(1)),
        }
    }

    fn poll_inner(&mut self, cx: &mut Context<'_>) -> Poll<Option<()>> {
        // Ensure the waker is up to date.
        if let Some(waker) = &mut self.flush {
            if !waker.will_wake(cx.waker()) {
                self.flush = Some(cx.waker().clone());
            }
        }

        if self.pending_value {
            return if self.flush.is_none() {
                // We have a queued value ready to be flushed.
                self.pending_value = false;
                Poll::Ready(Some(()))
            } else {
                // The async runtime would have no reason to be polling right now but we protect against it anyway.
                Poll::Pending
            };
        }

        match &mut self.inner {
            Inner::Dyn(v) => match v.as_mut().poll_next_value(cx) {
                Poll::Ready(v) => {
                    if self.flush.is_none() {
                        Poll::Ready(v)
                    } else {
                        match v {
                            Some(v) => {
                                self.pending_value = true;
                                Poll::Pending
                            }
                            None => Poll::Ready(None),
                        }
                    }
                }
                Poll::Pending => Poll::Pending,
            },
            Inner::Value(v) => {
                if self.flush.is_none() {
                    // Poll::Ready(v.take().map(Err))
                    todo!();
                } else {
                    Poll::Pending
                }
            }
        }
    }

    /// TODO
    pub fn poll_next(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<DynOutput<'_>, ProcedureError>>> {
        self.poll_inner(cx).map(|v| {
            v.map(|_: ()| {
                let Inner::Dyn(s) = &mut self.inner else {
                    unreachable!(); // TODO: Handle this?
                };
                s.as_mut().value()
            })
        })
    }

    /// TODO
    pub async fn next(&mut self) -> Option<Result<DynOutput<'_>, ProcedureError>> {
        poll_fn(|cx| self.poll_inner(cx)).await.map(|_: ()| {
            let Inner::Dyn(s) = &mut self.inner else {
                unreachable!(); // TODO: Handle this?
            };
            s.as_mut().value()
        })
    }

    /// TODO
    // TODO: Should error be `String` type?
    pub fn map<F: FnMut(Result<DynOutput, ProcedureError>) -> Result<T, String> + Unpin, T>(
        self,
        map: F,
    ) -> ProcedureStreamMap<F, T> {
        ProcedureStreamMap { stream: self, map }
    }
}

pub struct ProcedureStreamMap<
    F: FnMut(Result<DynOutput, ProcedureError>) -> Result<T, String> + Unpin,
    T,
> {
    stream: ProcedureStream,
    map: F,
}

impl<F: FnMut(Result<DynOutput, ProcedureError>) -> Result<T, String> + Unpin, T>
    ProcedureStreamMap<F, T>
{
    /// Start streaming data.
    /// Refer to `Self::require_manual_stream` for more information.
    pub fn stream(&mut self) {
        self.stream.stream();
    }

    /// Will return `true` if the future has resolved.
    ///
    /// For a stream created via `Self::from_future*` this will be `true` once the future has resolved and for all other streams this will always be `true`.
    pub fn resolved(&self) -> bool {
        self.stream.resolved()
    }

    /// Will return `true` if the stream is ready to start streaming data.
    ///
    /// This is `false` until the `flush` function is called by the user.
    pub fn flushable(&self) -> bool {
        self.stream.flushable()
    }
}

impl<F: FnMut(Result<DynOutput, ProcedureError>) -> Result<T, String> + Unpin, T> Stream
    for ProcedureStreamMap<F, T>
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        this.stream.poll_inner(cx).map(|v| {
            v.map(|_: ()| {
                let Inner::Dyn(s) = &mut this.stream.inner else {
                    unreachable!();
                };

                match (this.map)(s.as_mut().value()) {
                    Ok(v) => v,
                    // TODO: Exposing this error to the client or not?
                    // TODO: Error type???
                    Err(err) => todo!(),
                }
            })
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl fmt::Debug for ProcedureStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}

trait DynReturnValue: Send {
    fn poll_next_value<'a>(self: Pin<&'a mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>>;
    fn value(self: Pin<&mut Self>) -> Result<DynOutput<'_>, ProcedureError>;
    fn size_hint(&self) -> (usize, Option<usize>);
    fn resolved(&self) -> bool;
    fn flushed(&self) -> bool;
}

pin_project! {
    struct GenericDynReturnValue<S, T> {
        #[pin]
        inner: S,
        // `Stream::poll`
        poll: fn(Pin<&mut S>, &mut Context) -> Poll<Option<Result<T, ProcedureError>>>,
        // `Stream::size_hint`
        size_hint: fn(&S) -> (usize, Option<usize>),
        // convert the current value to a `DynOutput`
        as_value: fn(&mut Option<Result<T, ProcedureError>>) -> DynOutput<'_>,
        // detect when the stream has finished it's future if it has one.
        resolved: fn(&S) -> bool,
        // has the user called `flushed` within it?
        flushed: bool,
        // has the user panicked?
        unwound: bool,
        // the last yielded value. We place `T` here so we can type-erase it and avoiding boxing every value.
        // we hold `Result<_, ProcedureError>` for `ProcedureStream::require_manual_stream` to bepossible.
        // Be extemely careful changing this type as it's used in `DynOutput`'s downcasting!
        value: Option<Result<T, ProcedureError>>,
    }
}

impl<S: Send, T: Send> DynReturnValue for GenericDynReturnValue<S, T> {
    fn poll_next_value<'a>(mut self: Pin<&'a mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        if self.unwound {
            // The stream is now done.
            return Poll::Ready(None);
        }

        let this = self.as_mut().project();
        let r = catch_unwind(AssertUnwindSafe(|| {
            let _ = this.value.take(); // Reset value to ensure `take` being misused causes it to panic.
            (this.poll)(this.inner, cx).map(|v| {
                v.map(|v| {
                    *this.value = Some(v);
                    ()
                })
            })
        }));

        match r {
            Ok(v) => v,
            Err(err) => {
                *this.unwound = true;
                *this.value = Some(Err(ProcedureError::Unwind(err)));
                Poll::Ready(Some(()))
            }
        }
    }

    fn value(self: Pin<&mut Self>) -> Result<DynOutput<'_>, ProcedureError> {
        let this = self.project();
        match this.value {
            Some(Err(_)) => {
                let Some(Err(err)) = std::mem::replace(this.value, None) else {
                    unreachable!(); // checked above
                };
                Err(err)
            }
            v => Ok((this.as_value)(v)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size_hint)(&self.inner)
    }

    fn resolved(&self) -> bool {
        (self.resolved)(&self.inner)
    }
    fn flushed(&self) -> bool {
        self.flushed
    }
}
