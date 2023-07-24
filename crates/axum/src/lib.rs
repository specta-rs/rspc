//! Axum integration for rspc
#![allow(warnings)] // TODO: Remove once stabilized

// TODO: Crate lints

use std::{any::Any, sync::Arc};

use axum::{body::HttpBody, routing::any};
use rspc::{BuiltRouter, Router};

/// TODO
pub fn endpoint<S, B, TCtx: Send + Sync + 'static>(
    router: Arc<BuiltRouter<TCtx>>,
    ctx_fn: impl Any,
) -> axum::Router<S, B>
where
    B: HttpBody + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    axum::Router::new().route("/:id", any(|| async move { "Hello, world!" }))
}
