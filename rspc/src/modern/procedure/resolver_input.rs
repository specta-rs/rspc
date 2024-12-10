// /// The input to a procedure which is derived from an [`ProcedureInput`](crate::procedure::Argument).
// ///
// /// This trait has a built in implementation for any type which implements [`DeserializeOwned`](serde::de::DeserializeOwned).
// ///
// /// ## How this works?
// ///
// /// [`Self::from_value`] will be provided with a [`ProcedureInput`] which wraps the [`Argument::Value`](super::Argument::Value) from the argument provided to the [`Procedure::exec`](super::Procedure) call.
// ///
// /// Input is responsible for converting this value into the type the user specified for the procedure.
// ///
// /// If the type implements [`DeserializeOwned`](serde::de::DeserializeOwned) we will use Serde, otherwise we will attempt to downcast the value.
// ///
// /// ## Implementation for custom types
// ///
// /// Say you have a type `MyCoolThing` which you want to use as an argument to an rspc procedure:
// ///
// /// ```
// /// pub struct MyCoolThing(pub String);
// ///
// /// impl ResolverInput for MyCoolThing {
// ///     fn from_value(value: ProcedureInput<Self>) -> Result<Self, InternalError> {
// ///        Ok(todo!()) // Refer to ProcedureInput's docs
// ///     }
// /// }
// ///
// /// // You should also implement `ProcedureInput`.
// ///
// /// fn usage_within_rspc() {
// ///     <Procedure>::builder().query(|_, _: MyCoolThing| async move { () });
// /// }
// /// ```

// TODO: Should this be in `rspc_core`???
// TODO: Maybe rename?

use serde::de::DeserializeOwned;
use specta::{datatype::DataType, Type, TypeCollection};

/// TODO: Restore the above docs but check they are correct
pub trait ResolverInput: Sized + Send + 'static {
    fn data_type(types: &mut TypeCollection) -> DataType;

    /// Convert the [`DynInput`] into the type the user specified for the procedure.
    fn from_input(input: rspc_core::DynInput) -> Result<Self, rspc_core::ProcedureError>;
}

impl<T: DeserializeOwned + Type + Send + 'static> ResolverInput for T {
    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, specta::Generics::Definition)
    }

    fn from_input(input: rspc_core::DynInput) -> Result<Self, rspc_core::ProcedureError> {
        Ok(input.deserialize()?)
    }
}
