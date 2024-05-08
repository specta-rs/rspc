use serde::Serialize;

use super::output_value::ProcedureResult;

pub trait Output {
    fn into_result(self) -> ProcedureResult;
}

impl<T: Serialize + 'static> Output for T {
    fn into_result(self) -> ProcedureResult {
        ProcedureResult::with_serde(self)
    }
}

// TODO: Supporting regular streams?
