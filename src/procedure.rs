use specta::DataType;

use crate::FirstMiddleware;

pub struct ProcedureDataType{
    pub arg_ty: DataType,
    pub result_ty: DataType,
}

pub struct Procedure<TCtx> {
    pub exec: FirstMiddleware<TCtx>,
    pub ty: ProcedureDataType,
}
