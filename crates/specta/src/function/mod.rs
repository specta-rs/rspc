mod arg;
mod result;
#[cfg(feature = "tauri")]
mod tauri;

#[cfg(feature = "tauri")]
pub use self::tauri::*;
pub use arg::*;
pub use result::*;

use crate::*;

/// A helper for exporting the type of a Specta command.
///
/// ```rust
/// use specta::*;
///
/// #[specta]
/// fn some_function(name: String, age: i32) -> bool {
///     true
/// }
///
/// fn main() {
///      // This API is pretty new and will likely under go API changes in the future.
///      assert_eq!(
///         ts::export_datatype(&fn_datatype!(some_function).into()),
///         Ok("export type FunctionDataType = { name: \"some_function\", input: { name: string, age: number }, result: boolean }".to_string())
///      );
/// }
/// ```
#[macro_export]
macro_rules! fn_datatype {
    ($command:ident) => {{
        let mut type_map = ::specta::TypeDefs::default();
        ::specta::function::get_datatype_internal(
            $command as $crate::internal::_specta_paste! { [<__specta__ $command>]!(@signature) },
            $crate::internal::_specta_paste! { [<__specta__ $command>]!(@name) },
            &mut type_map,
            $crate::internal::_specta_paste! { [<__specta__ $command>]!(@arg_names) },
        )
    }};
    ($type_map:ident, $command:ident) => {{
        let mut type_map: ::specta::r#type::TypeDefs = $type_map;
        ::specta::command::get_datatype_internal(
            $command as $crate::internal::_specta_paste! { [<__specta__ $command>]!(@signature) },
            $crate::internal::_specta_paste! { [<__specta__ $command>]!(@name) },
            &mut $type_map,
            $crate::internal::_specta_paste! { [<__specta__ $command>]!(@arg_names) },
        )
    }};
}

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

/// is a helper for exporting a command to a `CommandDataType`. You shouldn't use this directly and instead should use [`export_fn!`](crate::export_fn).
pub fn get_datatype_internal<TMarker, T: SpectaFunction<TMarker>>(
    _: T,
    name: &'static str,
    type_map: &mut TypeDefs,
    fields: &[&'static str],
) -> FunctionDataType {
    T::to_datatype(name, type_map, fields)
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
