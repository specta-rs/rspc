mod arg;
mod result;
#[cfg(feature = "tauri")]
mod tauri;

#[cfg(feature = "tauri")]
pub use self::tauri::*;
pub use arg::*;
pub use result::*;

use crate::*;

/// Returns a [`FunctionDataType`] for a given function that has been annotated with
/// [`specta`](macro@crate::specta).
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
///     let typ = fn_datatype!(some_function);
///
///     assert_eq!(typ.name, "some_function");
///     assert_eq!(typ.args.len(), 2);
///     assert_eq!(typ.result, DataType::Primitive(PrimitiveType::bool))
/// }
/// ```
#[macro_export]
macro_rules! fn_datatype {
    ($function:path) => {{
        let mut type_map = $crate::TypeDefs::default();

        $crate::fn_datatype!(type_map, $function)
    }};
    ($type_map:ident, $function:path) => {{
        let type_map: &mut $crate::TypeDefs = &mut $type_map;

        $crate::internal::fn_datatype!(type_map, $function)
    }};
}

/// Contains type information about a function annotated with [`specta`](macro@crate::specta).
/// Returned by [`fn_datatype`].
#[derive(Debug, Clone)]
pub struct FunctionDataType {
    /// The name of the command. This will be derived from the Rust function name.
    pub name: &'static str,
    /// The input arguments of the command. The Rust functions arguments are converted into an [`DataType::Object`](crate::DataType::Object).
    pub args: Vec<(&'static str, DataType)>,
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
            args: vec![],
            result: TResult::to_datatype(DefOpts {
                parent_inline: false,
                type_map,
            }),
        }
    }
}

#[doc(hidden)]
/// is a helper for exporting a command to a `CommandDataType`. You shouldn't use this directly and instead should use [`fn_datatype!`](crate::fn_datatype).
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
            > SpectaFunction<(TResultMarker, $([<$i Marker>]),*)> for fn($($i),*) -> TResult {
                fn to_datatype(
                    name: &'static str,
                    type_map: &mut TypeDefs,
                    fields: &[&'static str],
                ) -> FunctionDataType {
                    let mut fields = fields.into_iter();

                    FunctionDataType {
                        name,
                        args: [$(
                            fields.next().and_then(|field|
                                $i::to_datatype(DefOpts {
                                    parent_inline: false,
                                    type_map,
                                })
                                .map(|ty| (*field, ty))
                            )
                        ),*,]
                        .into_iter()
                        .filter_map(|v| v)
                        .collect(),
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
