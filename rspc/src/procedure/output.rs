use serde::Serialize;

use super::result::ProcedureResult;

pub trait Output {
    fn result(self) -> ProcedureResult;
}

impl<T: Serialize + 'static> Output for T {
    fn result(self) -> ProcedureResult {
        ProcedureResult::with_serde(self)
    }
}

// TODO: Supporting regular streams?

// TODO: Break this outta the core.
pub struct SomeFile<T>(T);
// TODO: `std::any::Any` would be a `tokio::io::AsyncWrite` type thing.
impl<T: std::any::Any + 'static> Output for SomeFile<T> {
    fn result(self) -> ProcedureResult {
        let result: SomeFile<Box<dyn std::any::Any>> = SomeFile(Box::new(self.0));
        ProcedureResult::new(result)
    }
}
