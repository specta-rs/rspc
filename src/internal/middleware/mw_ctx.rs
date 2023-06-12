use std::borrow::Cow;

use serde_json::Value;

use super::{Executable2Placeholder, MwResultWithCtx};

pub struct MiddlewareContext {
    pub input: Value,
    pub req: RequestContext,
    // Prevents downstream user constructing type
    pub(crate) _priv: (),
}

impl MiddlewareContext {
    pub fn next<TNCtx>(self, ctx: TNCtx) -> MwResultWithCtx<TNCtx, Executable2Placeholder> {
        MwResultWithCtx {
            input: self.input,
            req: self.req,
            ctx: Some(ctx),
            resp: None,
        }
    }
}

/// TODO
// TODO: Is this a duplicate of any type?
// TODO: Move into public API cause it might be used in middleware
#[derive(Debug, Clone)]
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
    pub(crate) _priv: (),
}
