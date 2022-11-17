use std::future::Future;

use crate::{DataType, DefOpts, Type};

/// is a trait which is implemented by all types which can be used as a command result.
pub trait TypedCommandResult<TMarker> {
    /// convert result of the Rust function into a DataType
    fn to_datatype(opts: DefOpts) -> DataType;
}

#[cfg(feature = "serde")]
#[doc(hidden)]
pub enum TypedCommandResultSerialize {}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + Type> TypedCommandResult<TypedCommandResultSerialize> for T {
    fn to_datatype(opts: DefOpts) -> DataType {
        T::reference(opts, &[])
    }
}

#[doc(hidden)]
pub struct TypedCommandResultResult<TMarker>(TMarker);
impl<TMarker, T: TypedCommandResult<TMarker>, E>
    TypedCommandResult<TypedCommandResultResult<TMarker>> for Result<T, E>
{
    fn to_datatype(opts: DefOpts) -> DataType {
        T::to_datatype(opts)
    }
}

#[doc(hidden)]
pub struct TypedCommandResultFuture<TMarker>(TMarker);
impl<TMarker, T: TypedCommandResult<TMarker>, TFut: Future<Output = T>>
    TypedCommandResult<TypedCommandResultFuture<TMarker>> for TFut
{
    fn to_datatype(opts: DefOpts) -> DataType {
        T::to_datatype(opts)
    }
}
