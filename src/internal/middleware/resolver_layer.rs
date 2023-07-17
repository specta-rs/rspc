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
    type Stream<'a> = S;

    fn call(
        &self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        (self.func)(a, b, c)
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
