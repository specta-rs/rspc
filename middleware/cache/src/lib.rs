//! rspc-cache: Caching middleware for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

mod state;
mod store;

pub use state::CacheState;
pub use store::{Memory, Store};

use rspc::middleware::Middleware;

pub fn cache<TError, TCtx, TInput, TResult>() -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Clone + Send + 'static,
    TInput: Clone + Send + 'static,
    TResult: Send + 'static,
{
    Middleware::new(move |ctx: TCtx, input: TInput, next| async move {
        let cache = next.meta().state().get::<CacheState>().unwrap(); // TODO: Error handling

        // let cache = CacheState::builder(Arc::new(Memory)).mount();

        let result = next.exec(ctx, input).await;
        // TODO: Get cache tll
        // TODO: Use `Store`
        result
    })
}

/// Set the cache time-to-live (TTL) in seconds
pub fn cache_ttl(ttl: usize) {
    todo!();
}
