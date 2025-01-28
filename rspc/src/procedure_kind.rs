use std::fmt;

use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Type)]
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
