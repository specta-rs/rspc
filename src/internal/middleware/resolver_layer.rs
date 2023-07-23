use std::marker::PhantomData;

use futures::Stream;
use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;

use crate::{
    internal::{middleware::RequestContext, SealedLayer},
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
    type Stream<'a> = futures::future::Either<S, tracing_futures::Instrumented<S>>;

    #[cfg(not(feature = "tracing"))]
    type Stream<'a> = S;

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

        let result = (self.func)(
            ctx,
            serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
            req,
        );

        #[cfg(feature = "tracing")]
        drop(_enter);

        #[cfg(not(feature = "tracing"))]
        return result;

        #[cfg(feature = "tracing")]
        if let Some(span) = span {
            return result.map(|v| {
                futures::future::Either::Right(tracing_futures::Instrument::instrument(v, span))
            });
        } else {
            return result.map(futures::future::Either::Left);
        }
    }
}
