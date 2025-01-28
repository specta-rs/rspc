//! Internal types which power rspc. The module provides no guarantee of compatibility between updates, so you should be careful rely on types from it.

mod jsonrpc_exec;
mod middleware;
mod procedure_builder;
mod procedure_store;

pub(crate) use middleware::*;
pub(crate) use procedure_builder::*;
pub(crate) use procedure_store::*;

// Used by `rspc_axum`
pub use middleware::ProcedureKind;
pub mod jsonrpc;

// Were not exported by rspc 0.3.0 but required by `rspc::legacy` interop layer
#[doc(hidden)]
pub use middleware::{Layer, RequestContext, ValueOrStream};
