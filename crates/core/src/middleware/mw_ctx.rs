use std::{
    borrow::Cow,
    sync::{Arc, Mutex, PoisonError},
};

use serde_json::Value;

pub fn new_mw_ctx<TNCtx>(
    input: serde_json::Value,
    req: RequestContext,
    new_ctx: Arc<Mutex<Option<TNCtx>>>,
) -> MiddlewareContext<TNCtx> {
    MiddlewareContext {
        input,
        req,
        new_ctx,
    }
}

#[non_exhaustive]
pub struct MiddlewareContext<TNewCtx> {
    // From request
    pub input: Value,
    pub req: RequestContext,

    // For response
    new_ctx: Arc<Mutex<Option<TNewCtx>>>,
    // new_span: Option<tracing::Span>
}

impl<TNewCtx> MiddlewareContext<TNewCtx> {
    #[cfg(feature = "tracing")]
    pub fn with_span(mut self, span: Option<tracing::Span>) -> Self {
        self.req.span = Some(span);
        self
    }

    // TODO: Refactor return type
    pub fn next(self, ctx: TNewCtx) {
        self.new_ctx
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .replace(ctx);

        // TODO: Output of this
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
