use rspc_procedure::ProcedureError;
use specta::Type;

// TODO: Drop bounds on this cause they can be added at the impl.
pub trait Error: Type + 'static {
    fn into_procedure_error(self) -> ProcedureError;
}
