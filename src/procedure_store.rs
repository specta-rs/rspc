use crate::{internal::layer::DynLayer, router_builder::ProcedureDef};

// TODO: Rename this
#[deprecated]
pub struct ProcedureTodo<TCtx> {
    // TODO: Back to `pub(crate)`
    pub exec: Box<dyn DynLayer<TCtx>>,
    pub ty: ProcedureDef,
}
