use std::marker::PhantomData;

use futures::Stream;
use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;

use crate::{
    internal::{middleware::RequestContext, DynBody, SealedLayer},
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
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
        body: &mut DynBody,
    ) -> Result<Self::Stream<'_>, ExecError> {
        #[cfg(feature = "tracing")]
        let span = c.span();

        #[cfg(feature = "tracing")]
        let _enter = span.as_ref().map(|s| s.enter());

        // TODO: Somehow get `B` type which was built from the many content types into here (where it will wrap the upstream `B` webserver body generic)

        // TODO: Poll the content (with the incoming body) until `TArg` is yielded

        let result = (self.func)(
            a,
            // TODO: Using content type system
            serde_json::from_value(b).map_err(ExecError::DeserializingArgErr)?,
            c,
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
