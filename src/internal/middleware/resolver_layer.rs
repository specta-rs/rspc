use std::{
    cell::RefCell,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use bytes::Bytes;
use futures::Stream;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;

use crate::{
    internal::{exec::RspcStream, middleware::RequestContext, SealedLayer},
    ExecError,
};

pub(crate) struct ResolverLayer<T, TArg> {
    func: T,
    phantom: PhantomData<fn() -> TArg>,
}

impl<T, TArg> ResolverLayer<T, TArg> {
    pub(crate) fn new(func: T) -> Self {
        Self {
            func,
            phantom: PhantomData,
        }
    }
}

// TODO: For `T: ResolverFunction` or something like that to simplify the generics
impl<T, TArg, TLayerCtx, S> SealedLayer<TLayerCtx> for ResolverLayer<T, TArg>
where
    TLayerCtx: Send + Sync + 'static,
    TArg: Type + DeserializeOwned + 'static,
    T: Fn(TLayerCtx, TArg, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
    S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
{
    #[cfg(feature = "tracing")]
    type Stream<'a> = DecodeBody<futures::future::Either<S, tracing_futures::Instrumented<S>>>;

    #[cfg(not(feature = "tracing"))]
    type Stream<'a> = DecodeBody<S>;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        #[cfg(feature = "tracing")]
        let span = req.span();

        #[cfg(feature = "tracing")]
        let _enter = span.as_ref().map(|s| s.enter());

        // TODO: Using content types lol
        let input = serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?;
        let result = (self.func)(ctx, input, req);

        #[cfg(feature = "tracing")]
        drop(_enter);

        #[cfg(not(feature = "tracing"))]
        return result.map(|stream| DecodeBody {
            got_body: false,
            stream,
        });

        #[cfg(feature = "tracing")]
        return if let Some(span) = span {
            result.map(|v| {
                futures::future::Either::Right(tracing_futures::Instrument::instrument(v, span))
            })
        } else {
            result.map(futures::future::Either::Left)
        }
        .map(|stream| DecodeBody {
            got_body: false,
            stream,
        });
    }
}

pub(crate) struct State {
    pub waker: Option<Waker>,
    pub chunk: Option<Bytes>,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: None,
            chunk: None,
        }
    }
}

thread_local!(pub(crate) static STATE: RefCell<State> = RefCell::new(State::new()));

pin_project! {
    /// TODO
    // TODO: Seal this
    pub struct DecodeBody<S> {
        got_body: bool,
        #[pin]
        stream: S
    }
}

impl<S: Stream> RspcStream for DecodeBody<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if !*this.got_body {
            *this.got_body = true;
            println!("GET BODY");

            STATE.with(|v| {
                v.borrow_mut().waker = Some(cx.waker().clone());
            });

            return Poll::Pending;
        } else {
            println!("GOT BODY {:?}", STATE.with(|v| v.borrow_mut().chunk.take()));
        }

        this.stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
