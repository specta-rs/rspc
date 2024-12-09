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

use std::{
    cell::Cell,
    future::{poll_fn, Future},
    pin::pin,
};

pub use memory::Memory;
pub use state::CacheState;
pub use store::Store;

use rspc::middleware::Middleware;
use store::Value;

thread_local! {
    static CACHE_TTL: Cell<Option<usize>> = Cell::new(None);
}

/// Set the cache time-to-live (TTL) in seconds
pub fn cache_ttl(ttl: usize) {
    CACHE_TTL.set(Some(ttl));
}

pub fn cache<TError, TCtx, TInput, TResult>() -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Send + 'static,
    TInput: Clone + Send + 'static,
    TResult: Clone + Send + Sync + 'static,
{
    // let todo = poll_fn(|cx| {
    //     todo!();
    // });

    Middleware::new(move |ctx: TCtx, input: TInput, next| {
        async move {
            let meta = next.meta();
            let cache = meta.state().get::<CacheState>().unwrap(); // TODO: Error handling

            let key = "todo"; // TODO: Work this out properly
                              // TODO: Keyed to `TInput`

            if let Some(value) = cache.store().get(key) {
                let value: &TResult = value.downcast_ref().unwrap(); // TODO: Error
                return Ok(value.clone());
            }

            let fut = next.exec(ctx, input);
            let mut fut = pin!(fut);

            let (ttl, result): (Option<usize>, Result<TResult, TError>) =
                poll_fn(|cx| fut.as_mut().poll(cx).map(|v| (CACHE_TTL.get(), v))).await;

            // self.project().fut.poll(cx).map(|v| {
            //     if let Some(ttl) = CACHE_TTL.get() {
            //         println!("REQUESTED CACHE {:?}", ttl);
            //     }

            //     v
            // })

            // TODO: Only cache if `ttl` is actually set
            if let Some(ttl) = CACHE_TTL.get() {
                // TODO: Caching error responses?
                if let Ok(value) = &result {
                    cache.store().set(key, Value::new(value.clone()), ttl);
                };
            }

            result
        }
    })
}

// pin_project! {
//     struct Cached<F> {
//         #[pin]
//         fut: F,
//     }
// }

// impl<F: Future> Future for Cached<F> {
//     type Output = F::Output;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         // match self.project().fut.poll(cx) {
//         //     Poll::Ready(_) => todo!(),
//         //     Poll::Pending => todo!(),
//         // }

//         self.project().fut.poll(cx).map(|v| {
//             if let Some(ttl) = CACHE_TTL.get() {
//                 println!("REQUESTED CACHE {:?}", ttl);
//             }

//             v
//         })
//     }
// }
