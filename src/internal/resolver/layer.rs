use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;
use std::marker::PhantomData;

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
    S: Body + Send + 'static,
{
    type Stream<'a> = S;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        // TODO: Using content types lol
        let input = serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?;
        return (self.func)(ctx, input, req);
    }
}
