use specta::Typedef;

use crate::FirstMiddleware;

pub struct ProcedureTypedef {
    pub arg_ty: Typedef,
    pub result_ty: Typedef,
}

pub struct Procedure<TCtx> {
    pub exec: FirstMiddleware<TCtx>,
    pub ty: ProcedureTypedef,
}
