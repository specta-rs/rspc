use crate::utils::format_fn_wrapper;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Path, Token,
};

pub struct FnDatatypeInput {
    type_map: Ident,
    function: Path,
}

impl Parse for FnDatatypeInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let type_map: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let function: Path = input.parse()?;

        Ok(Self { type_map, function })
    }
}

pub fn proc_macro(FnDatatypeInput { type_map, function }: FnDatatypeInput) -> TokenStream {
    let mut specta_fn_macro = function.clone();

    let last = specta_fn_macro
        .segments
        .last_mut()
        .expect("Function path is empty!");

    last.ident = format_fn_wrapper(&last.ident.clone());

    let fn_signature = quote!(#specta_fn_macro!(@signature));
    let fn_name = quote!(#specta_fn_macro!(@name));
    let fn_arg_names = quote!(#specta_fn_macro!(@arg_names));

    quote! {
        specta::function::get_datatype_internal(
            #function as #fn_signature,
            #fn_name,
            #type_map,
            #fn_arg_names
        )
    }
}
