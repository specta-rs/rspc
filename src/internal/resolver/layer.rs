use futures::Stream;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    internal::{middleware::RequestContext, Body, SealedLayer},
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
    S: Body + Send + 'static, // Stream<Item = Result<Value, ExecError>> + Send + 'static,
{
    // #[cfg(feature = "tracing")]
    // type Stream<'a> = futures::future::Either<S, tracing_futures::Instrumented<S>>;

    // #[cfg(not(feature = "tracing"))]
    type Stream<'a> = S;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        // #[cfg(feature = "tracing")]
        // let span = req.span();

        // #[cfg(feature = "tracing")]
        // let _enter = span.as_ref().map(|s| s.enter());

        // // TODO: Using content types lol
        // let input = serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?;
        // let result = (self.func)(ctx, input, req);

        // #[cfg(feature = "tracing")]
        // drop(_enter);

        // #[cfg(not(feature = "tracing"))]
        // return result.map(|stream| DecodeBody { stream });

        // #[cfg(feature = "tracing")]
        // return if let Some(span) = span {
        //     result.map(|v| {
        //         futures::future::Either::Right(tracing_futures::Instrument::instrument(v, span))
        //     })
        // } else {
        //     result.map(futures::future::Either::Left)
        // }
        // .map(|stream| DecodeBody { stream });
        todo!();
    }
}
