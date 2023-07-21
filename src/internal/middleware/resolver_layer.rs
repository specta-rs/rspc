use futures::Stream;
use serde_json::Value;

use crate::{
    internal::{middleware::RequestContext, SealedLayer},
    ExecError,
};

pub(crate) struct ResolverLayer<T> {
    func: T,
}

impl<T> ResolverLayer<T> {
    pub(crate) fn new(func: T) -> Self {
        Self { func }
    }
}

// TODO: For `T: ResolverFunction` or something like that to simplify the generics
impl<T, TLayerCtx, S> SealedLayer<TLayerCtx> for ResolverLayer<T>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
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
    ) -> Result<Self::Stream<'_>, ExecError> {
        #[cfg(feature = "tracing")]
        let span = c.span.clone();

        #[cfg(feature = "tracing")]
        let _enter = span.as_ref().map(|s| s.enter());

        let result = (self.func)(a, b, c);

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

// mod private {
//     use futures::Stream;
//     use serde_json::Value;

//     use crate::{
//         internal::{middleware::RequestContext, SealedLayer},
//         ExecError,
//     };

//     // TODO: For `T: ResolverFunction` or something like that to simplify the generics
//     pub struct ResolverLayer<T> {
//         func: T,
//     }

//     impl<T> ResolverLayer<T> {
//         pub(crate) fn new(func: T) -> Self {
//             Self { func }
//         }
//     }

//     impl<T, TLayerCtx, S> SealedLayer<TLayerCtx> for ResolverLayer<T>
//     where
//         TLayerCtx: Send + Sync + 'static,
//         T: Fn(TLayerCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
//         S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
//     {
//         type Stream<'a> = S;

//         fn call(
//             &self,
//             a: TLayerCtx,
//             b: Value,
//             c: RequestContext,
//         ) -> Result<Self::Stream<'_>, ExecError> {
//             (self.func)(a, b, c)
//         }
//     }
// }

// pub(crate) use private::ResolverLayer;
