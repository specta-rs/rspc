use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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

    let function_name = specta_fn_macro
        .segments
        .pop()
        .expect("Function path is empty!")
        .into_value();

    let specta_fn_ident = format_ident!("__specta__fn__{}", function_name.ident);

    specta_fn_macro.segments.push(syn::PathSegment {
        ident: specta_fn_ident,
        ..function_name
    });

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
