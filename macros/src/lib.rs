use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Key)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("trpc_rs");
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let mut variants = Vec::new();
    match data {
        Data::Enum(data) => {
            for variant in data.variants {
                let variant_ident = &variant.ident;
                let variant_string = variant_ident.to_string().to_case(Case::Camel);
                variants.push(quote! { #ident::#variant_ident => #variant_string });
            }
        }
        _ => panic!("The 'Key' derive macro is only supported on enums!"),
    }

    quote! {
        impl #crate_name::Key for #ident {
            fn to_val(&self) -> &'static str {
                match self {
                    #(#variants),*
                }
            }
        }

        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_val())
            }
        }
    }
    .into()
}
