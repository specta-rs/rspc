use std::future::Future;

use crate::{DataType, DefOpts, Type};

/// is a trait which is implemented by all types which can be used as a command result.
pub trait SpectaFunctionResult<TMarker> {
    /// convert result of the Rust function into a DataType
    fn to_datatype(opts: DefOpts) -> DataType;
}

#[cfg(feature = "serde")]
#[doc(hidden)]
pub enum SpectaFunctionResultSerialize {}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + Type> SpectaFunctionResult<SpectaFunctionResultSerialize> for T {
    fn to_datatype(opts: DefOpts) -> DataType {
        T::reference(opts, &[])
    }
}

#[doc(hidden)]
pub struct SpectaFunctionResultResult<TMarker>(TMarker);
impl<TMarker, T: SpectaFunctionResult<TMarker>, E>
    SpectaFunctionResult<SpectaFunctionResultResult<TMarker>> for Result<T, E>
{
    fn to_datatype(opts: DefOpts) -> DataType {
        T::to_datatype(opts)
    }
}

#[doc(hidden)]
pub struct SpectaFunctionResultFuture<TMarker>(TMarker);
impl<TMarker, T: SpectaFunctionResult<TMarker>, TFut: Future<Output = T>>
    SpectaFunctionResult<SpectaFunctionResultFuture<TMarker>> for TFut
{
    fn to_datatype(opts: DefOpts) -> DataType {
        T::to_datatype(opts)
    }
}
