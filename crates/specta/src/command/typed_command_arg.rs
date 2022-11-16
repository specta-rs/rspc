use crate::{DataType, DefOpts, Type};

pub trait TypedCommandArg<TMarker> {
    fn to_datatype(opts: DefOpts) -> Option<DataType>;
}

pub enum TypedCommandArgDeserializeMarker {}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de> + Type> TypedCommandArg<TypedCommandArgDeserializeMarker>
    for T
{
    fn to_datatype(opts: DefOpts) -> Option<DataType> {
        Some(T::reference(opts, &[]))
    }
}
