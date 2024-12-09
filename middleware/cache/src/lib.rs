//! rspc-cache: Caching middleware for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

mod memory;
mod state;
mod store;

pub use memory::Memory;
pub use state::CacheState;
pub use store::Store;

use rspc::middleware::Middleware;
use store::Value;

pub fn cache<TError, TCtx, TInput, TResult>() -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Send + 'static,
    TInput: Clone + Send + 'static,
    TResult: Clone + Send + Sync + 'static,
{
    Middleware::new(move |ctx: TCtx, input: TInput, next| async move {
        let meta = next.meta();
        let cache = meta.state().get::<CacheState>().unwrap(); // TODO: Error handling

        let key = "todo"; // TODO: Work this out properly
                          // TODO: Keyed to `TInput`

        if let Some(value) = cache.store().get(key) {
            let value: &TResult = value.downcast_ref().unwrap(); // TODO: Error
            return Ok(value.clone());
        }

        let result: Result<TResult, TError> = next.exec(ctx, input).await;

        // TODO: Caching error responses?
        if let Ok(value) = &result {
            // TODO: Get ttl from `cache_tll`
            cache.store().set(key, Value::new(value.clone()), 0);
        };

        result
    })
}

/// Set the cache time-to-live (TTL) in seconds
pub fn cache_ttl(ttl: usize) {
    // TODO: Implement
}
