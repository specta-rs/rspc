//! Easily export your Rust types to other languages
//!
//! This crate contains the macro which are reexported by the `specta` crate.
//! You shouldn't need to use this crate directly.
//! Checkout [Specta](https://docs.rs/specta).
//!

use syn::parse_macro_input;
#[macro_use]
mod utils;
mod data_type_from;
mod fn_datatype;
mod specta;
mod r#type;

#[proc_macro_derive(Type, attributes(specta, serde, doc))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r#type::derive(input, "specta".into()).unwrap_or_else(|err| err.into_compile_error().into())
}

/// This macro is exposed from rspc as a wrapper around [Type] with a correct import path.
/// This is exposed from here so rspc doesn't need a macro package for 4 lines of code.
#[doc(hidden)]
#[proc_macro_derive(RSPCType, attributes(specta, serde, doc))]
pub fn derive_rspc_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r#type::derive(input, "rspc::internal::specta".into())
        .unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_derive(DataTypeFrom, attributes(specta))]
pub fn derive_data_type_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    data_type_from::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_attribute]
pub fn specta(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    specta::attribute(item).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro]
pub fn fn_datatype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fn_datatype::proc_macro(parse_macro_input!(input as fn_datatype::FnDatatypeInput))
        .unwrap_or_else(|err| err.into_compile_error().into())
        .into()
}
