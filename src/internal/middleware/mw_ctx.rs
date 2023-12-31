use std::{
    borrow::Cow,
    marker::PhantomData,
    sync::{Arc, Mutex, PoisonError},
};

use serde_json::Value;

use crate::internal::layer::{middleware_layer_stream::get_next_stream, NextStream};

pub(crate) fn new_mw_ctx<TNewCtx>(
    input: serde_json::Value,
    req: RequestContext,
) -> MiddlewareContext<TNewCtx> {
    MiddlewareContext {
        input,
        req,
        phantom: PhantomData,
    }
}

#[non_exhaustive]
pub struct MiddlewareContext<TNewCtx> {
    pub input: Value,
    pub req: RequestContext,
    phantom: PhantomData<TNewCtx>,
}

impl<TNewCtx: Send + 'static> MiddlewareContext<TNewCtx> {
    #[cfg(feature = "tracing")]
    pub fn with_span(mut self, span: Option<tracing::Span>) -> Self {
        self.req.span = Some(span);
        self
    }

    pub async fn next(self, ctx: TNewCtx) -> NextStream {
        get_next_stream(ctx, self.input, self.req).await
    }
}

/// TODO
// TODO: Is this a duplicate of any type?
// TODO: Move into public API cause it might be used in middleware
#[derive(Debug, Clone, Copy)]
pub enum ProcedureKind {
    Query,
    Mutation,
    Subscription,
}

impl ProcedureKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            ProcedureKind::Query => "query",
            ProcedureKind::Mutation => "mutation",
            ProcedureKind::Subscription => "subscription",
        }
    }
}

/// TODO
// TODO: Maybe rename to `Request` or something else. Also move into Public API cause it might be used in middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub id: u32,
    pub kind: ProcedureKind,
    pub path: Cow<'static, str>,
    #[cfg(feature = "tracing")]
    span: Option<Option<tracing::Span>>,
    // Prevents downstream user constructing type
    _priv: (),
}

impl RequestContext {
    pub(crate) fn new(id: u32, kind: ProcedureKind, path: Cow<'static, str>) -> Self {
        Self {
            id,
            #[cfg(feature = "tracing")]
            span: None,
            kind,
            path,
            _priv: (),
        }
    }

    #[cfg(feature = "tracing")]
    pub fn span(&self) -> Option<tracing::Span> {
        self.span.clone().unwrap_or_else(|| {
            Some(match self.kind {
                ProcedureKind::Query => {
                    let query = self.path.as_ref();
                    tracing::info_span!("rspc", query)
                }
                ProcedureKind::Mutation => {
                    let mutation = self.path.as_ref();
                    tracing::info_span!("rspc", mutation)
                }
                ProcedureKind::Subscription => {
                    let subscription = self.path.as_ref();
                    tracing::info_span!("rspc", subscription)
                }
            })
        })
    }
}
