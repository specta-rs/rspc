//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc

mod builder;
mod input;
mod input_value;
mod output;
mod output_value;
mod procedure;
mod stream;

pub use builder::ProcedureBuilder;
pub use input::Input;
pub use input_value::InputValue;
pub use output::Output;
pub use output_value::ProcedureResult;
pub use procedure::Procedure;
pub use stream::ProcedureStream;

// TODO: Remove this, it's just as a prototype
#[doc(hidden)]
pub struct File<T = Box<dyn tokio::io::AsyncWrite>>(T);
impl<T: tokio::io::AsyncWrite + 'static> Output for File<T> {
    fn into_result(self) -> ProcedureResult {
        let result: File<Box<dyn tokio::io::AsyncWrite>> = File(Box::new(self.0));
        ProcedureResult::new(result)
    }
}
impl<T: tokio::io::AsyncWrite + 'static> Input for File<T> {
    type Input = ();

    fn deserialize(self) -> Option<Self::Input> {
        None
    }

    fn from_value(value: InputValue) -> Option<Self> {
        value.downcast().map(Self)
    }
}
