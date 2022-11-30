#[cfg(feature = "tauri")]
mod tauri;
mod typed_command;
mod typed_command_arg;
mod typed_command_result;

#[cfg(feature = "tauri")]
pub use self::tauri::*;
pub use typed_command::*;
pub use typed_command_arg::*;
pub use typed_command_result::*;

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
