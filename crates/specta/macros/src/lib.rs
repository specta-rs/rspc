#[macro_use]
mod utils;
mod command;
mod to_data_type;
mod r#type;

#[proc_macro_derive(Type, attributes(specta, serde))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r#type::derive(input, "specta".into())
}

/// This macro is exposed from rspc as a wrapper around [Type] with a correct import path.
/// This is exposed from here so rspc doesn't need a macro package for 4 lines of code.
#[doc(hidden)]
#[proc_macro_derive(RSPCType, attributes(specta, serde))]
pub fn derive_rspc_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r#type::derive(input, "rspc::internal::specta".into())
}

#[proc_macro_derive(ToDataType, attributes(specta))]
pub fn derive_to_data_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    to_data_type::derive(input)
}

#[proc_macro_attribute]
pub fn command(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    command::command(item)
}
