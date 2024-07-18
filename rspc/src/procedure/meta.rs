use core::fmt;
use std::{borrow::Cow, collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, specta::Type)]
#[specta(rename_all = "camelCase")]
pub enum ProcedureKind {
    Query,
    Mutation,
    Subscription,
}

impl fmt::Display for ProcedureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query => write!(f, "Query"),
            Self::Mutation => write!(f, "Mutation"),
            Self::Subscription => write!(f, "Subscription"),
        }
    }
}

#[derive(Debug, Clone)]
enum ProcedureName {
    Static(&'static str),
    Dynamic(Arc<String>),
}

#[derive(Debug, Clone)]
pub struct ProcedureMeta {
    name: ProcedureName,
    kind: ProcedureKind,
}

impl ProcedureMeta {
    pub(crate) fn new(name: Cow<'static, str>, kind: ProcedureKind) -> Self {
        Self {
            name: ProcedureName::Dynamic(Arc::new(name.into_owned())),
            kind,
        }
    }
}

impl ProcedureMeta {
    pub fn name(&self) -> &str {
        match &self.name {
            ProcedureName::Static(name) => name,
            ProcedureName::Dynamic(name) => name.as_str(),
        }
    }

    pub fn kind(&self) -> ProcedureKind {
        self.kind
    }
}
