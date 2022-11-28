use super::{SpectaFunctionArg, SpectaFunctionResult};
use crate::{DataType, DataTypeFrom, DefOpts, ObjectField, ObjectType, TypeDefs};

/// is a struct which represents the datatype of a Specta command.
#[derive(Debug, DataTypeFrom)]
#[specta(crate = "crate")]
pub struct FunctionDataType {
    /// The name of the command. This will be derived from the Rust function name.
    pub name: &'static str,
    /// The input arguments of the command. The Rust functions arguments are converted into an [`DataType::Object`](crate::DataType::Object).
    pub input: Option<DataType>,
    /// The result type of the command. This would be the return type of the Rust function.
    pub result: DataType,
}

/// is a trait which is implemented by all functions which can be used as a command.
pub trait SpectaFunction<TMarker> {
    /// convert function into a DataType
    fn to_datatype(
        name: &'static str,
        type_map: &mut TypeDefs,
        fields: &[&'static str],
    ) -> FunctionDataType;
}

impl<TResultMarker, TResult: SpectaFunctionResult<TResultMarker>> SpectaFunction<TResultMarker>
    for fn() -> TResult
{
    fn to_datatype(
        name: &'static str,
        type_map: &mut TypeDefs,
        _fields: &[&'static str],
    ) -> FunctionDataType {
        FunctionDataType {
            name,
            input: None,
            result: TResult::to_datatype(DefOpts {
                parent_inline: false,
                type_map,
            }),
        }
    }
}

macro_rules! impl_typed_command {
    ( impl $($i:ident),* ) => {
       paste::paste! {
            impl<
                    TResultMarker,
                    TResult: SpectaFunctionResult<TResultMarker>,
                    $([<$i Marker>]),*,
                    $($i: SpectaFunctionArg<[<$i Marker>]>),*
                > SpectaFunction<(TResultMarker, $([<$i Marker>]),*)> for fn($($i),*) -> TResult
            {
                fn to_datatype(
                    name: &'static str,
                    type_map: &mut TypeDefs,
                    fields: &[&'static str],
                ) -> FunctionDataType {
                    let mut fields = fields.into_iter();

                    FunctionDataType {
                        name,
                        input: Some(DataType::Object(ObjectType {
                            name: "_unreachable_".into(),
                            generics: vec![],
                            fields: [
                                $(
                                    $i::to_datatype(DefOpts {
                                        parent_inline: false,
                                        type_map,
                                    })
                                    .map(|ty| ObjectField {
                                        name: fields.next().expect("Tauri Specta reached an unreachable state. The macro returns the incorrect number of fields. Please file this as a bug on GitHub!").to_string(),
                                        ty: ty,
                                        optional: false,
                                        flatten: false,
                                    })
                                ),*,
                            ]
                            .into_iter()
                            .filter_map(|v| v)
                            .collect(),
                            tag: None,
                            type_id: None,
                        })),
                        result: TResult::to_datatype(DefOpts {
                            parent_inline: false,
                            type_map,
                        }),
                    }
                }
            }
        }
    };
    ( $i2:ident $(, $i:ident)* ) => {
        impl_typed_command!(impl $i2 $(, $i)* );
        impl_typed_command!($($i),*);
    };
    () => {};
}

impl_typed_command!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

/// is a helper for exporting a command to a `CommandDataType`. You shouldn't use this directly and instead should use [`export_fn!`](crate::export_fn).
pub fn get_datatype_internal<TMarker, T: SpectaFunction<TMarker>>(
    _: T,
    name: &'static str,
    type_map: &mut TypeDefs,
    fields: &[&'static str],
) -> FunctionDataType {
    T::to_datatype(name, type_map, fields)
}
