//! Integrate rspc with an [Axum](https://docs.rs/axum/latest/axum/) HTTP server so it can be accessed from your frontend.
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]
#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use axum::{
    extract::Request,
    http::{Method, StatusCode},
    routing::{on, MethodFilter},
    Router,
};
use rspc_core::IntoRouter;

// TODO: Async context function
// TODO: Allow context function to access request

pub fn endpoint<R: IntoRouter>(router: R, ctx_fn: impl Fn() -> R::Ctx) -> Router {
    let executor = router.build();
    Router::new().route(
        "/:id",
        on(
            MethodFilter::GET.or(MethodFilter::POST),
            |req: Request| async move {
                match (req.method(), &req.uri().path()[1..]) {
                    (&Method::GET, "ws") => todo!(), // handle_websocket(router, ctx_fn, req).into_response(),
                    (&Method::GET, _) => todo!(), // handle_http(router, ctx_fn, req).await.into_response(),
                    (&Method::POST, "_batch") => todo!(), // handle_http_batch(router, ctx_fn, req).await.into_response()
                    (&Method::POST, _) => todo!(), // handle_http(router, ctx_fn, req).await.into_response(),
                    _ => (StatusCode::METHOD_NOT_ALLOWED, Vec::<u8>::new()),
                }
            },
        ),
    )
}
