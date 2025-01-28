use rspc_procedure::ProcedureError;

// TODO: Drop bounds on this cause they can be added at the impl.
pub trait Error: 'static {
    fn into_procedure_error(self) -> ProcedureError;
}
