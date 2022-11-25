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
/// use specta::{export_fn, ts::ts_export_datatype};
///
/// #[specta::command]
/// fn some_function(name: String, age: i32) -> bool {
///     true
/// }
///
/// fn main() {
///      // This API is pretty new and will likely under go API changes in the future.
///      assert_eq!(
///         ts_export_datatype(&export_fn!(some_function).into()),
///         Ok("export interface CommandDataType { name: \"some_function\", input: { name: string, age: number }, result: boolean }".to_string())
///      );
/// }
/// ```

#[macro_export]
macro_rules! export_fn {
    ($command:ident) => {{
        let mut type_map = ::specta::TypeDefs::default();
        ::specta::command::export_command_datatype(
            $command
                as $crate::internal::_specta_paste! { [<__specta__cmd__ $command>]!(@signature) },
            $crate::internal::_specta_paste! { [<__specta__cmd__ $command>]!(@name) },
            &mut type_map,
            $crate::internal::_specta_paste! { [<__specta__cmd__ $command>]!(@arg_names) },
        )
    }};
    ($type_map:ident, $command:ident) => {{
        let mut type_map: ::specta::r#type::TypeDefs = $type_map;
        ::specta::command::export_command_datatype(
            $command
                as $crate::internal::_specta_paste! { [<__specta__cmd__ $command>]!(@signature) },
            $crate::internal::_specta_paste! { [<__specta__cmd__ $command>]!(@name) },
            &mut $type_map,
            $crate::internal::_specta_paste! { [<__specta__cmd__ $command>]!(@arg_names) },
        )
    }};
}
