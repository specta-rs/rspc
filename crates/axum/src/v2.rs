use axum::Router;
use rspc_core::Procedures;

use crate::extractors::TCtxFunc;

pub fn endpoint2<TCtx, TCtxFnMarker, TCtxFn, S>(
    router: impl Into<Procedures<TCtx>>,
    ctx_fn: TCtxFn,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TCtxFnMarker: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, S, TCtxFnMarker>,
{
    let flattened = router
        .into()
        .into_iter()
        .map(|(key, value)| (key.join("."), value));

    // TODO: Flatten keys

    todo!();
}
