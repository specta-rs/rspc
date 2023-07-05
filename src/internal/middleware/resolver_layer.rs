mod private {
    use std::marker::PhantomData;

    use futures::Stream;
    use serde_json::Value;

    use crate::{
        internal::{middleware::RequestContext, SealedLayer},
        ExecError,
    };

    // TODO: For `T: ResolverFunction` or something like that to simplify the generics
    pub struct ResolverLayer<TLayerCtx, T, S>
    where
        TLayerCtx: Send + Sync + 'static,
        T: Fn(TLayerCtx, Value, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
        S: Stream<Item = Result<Value, ExecError>> + Send + 'static,
    {
        pub func: T,
        pub phantom: PhantomData<TLayerCtx>,
    }

    impl<T, TLayerCtx, S> SealedLayer<TLayerCtx> for ResolverLayer<TLayerCtx, T, S>
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
}

pub(crate) use private::ResolverLayer;
