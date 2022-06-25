use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Key)]
pub fn derive_key(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("trpc_rs");
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let mut variants = Vec::new();
    match data {
        Data::Enum(data) => {
            for variant in data.variants {
                let variant_ident = &variant.ident;
                let variant_string = variant_ident.to_string().to_case(Case::Camel);

                match variant.fields {
                    Fields::Unit => {}
                    _ => {
                        panic!("The 'Key' derive macros requires all enum variants to be a unit variant (hold no value).");
                    }
                }

                variants.push(quote! { #ident::#variant_ident => #variant_string.into() });
            }
        }
        _ => panic!("The 'Key' derive macro is only supported on enums!"),
    }

    quote! {
        impl #crate_name::KeyDefinition for #ident {
           type Key = #ident;
        }

        impl<TArg> #crate_name::Key<#ident, TArg> for #ident {
            type Arg = TArg;

            fn to_val(&self) -> String {
                match self {
                    #(#variants),*
                }
            }
        }
    }
    .into()
}

#[derive(FromDeriveInput)]
#[darling(attributes(query), forward_attrs(allow, doc, cfg))]
struct Args {
    key: Option<syn::Ident>,
}

#[proc_macro_derive(Query, attributes(query))]
pub fn derive_query(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("trpc_rs");
    let input: DeriveInput = parse_macro_input!(input);
    let args = match Args::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };
    let DeriveInput { ident, data, .. } = input;
    let key_ident = args.key.unwrap_or(format_ident!("QueryKey"));
    let key_wrapper = format_ident!("{}KeyWrapper", ident);

    let mut key_consts = Vec::new();
    match data {
        Data::Enum(data) => {
            for variant in data.variants {
                let variant_ident = variant.ident;
                let variant_string = variant_ident.to_string().to_case(Case::Camel);

                let variant_ty = match variant.fields {
                    Fields::Named(_) => {
                        panic!("The 'Query' derive macros does not support named fields.")
                    }
                    Fields::Unnamed(fields) => match fields.unnamed.len() {
                        0 => quote! { () },
                        1 => {
                            let field_ty = fields.unnamed[0].ty.clone();
                            quote! { #field_ty}
                        }
                        _ => {
                            panic!("The 'Query' derive macro requires all enum variants to have at most one unnamed field.");
                        }
                    },
                    Fields::Unit => quote! { () },
                };
                key_consts.push(quote! { const #variant_ident: #key_wrapper<#variant_ty> = #key_wrapper(#variant_string, std::marker::PhantomData); });
            }
        }
        _ => panic!("The 'Key' derive macro is only supported on enums!"),
    }

    quote! {
        impl #crate_name::KeyDefinition for #ident {
            type Key = #key_ident;
        }

        #[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
        pub struct #key_ident;

        pub struct #key_wrapper<TArg>(&'static str, std::marker::PhantomData<TArg>);

        impl<TArg> #crate_name::Key<#key_ident, TArg> for #key_wrapper<TArg> {
            type Arg = TArg;

            fn to_val(&self) -> String {
                self.0.into()
            }
        }

        #[allow(non_upper_case_globals)]
        impl #key_ident {
            #(#key_consts)*
        }
    }
    .into()
}

#[proc_macro_derive(Mutation, attributes(query))]
pub fn derive_mutation(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("trpc_rs");
    let input: DeriveInput = parse_macro_input!(input);
    let args = match Args::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };
    let DeriveInput { ident, data, .. } = input;
    let key_ident = args.key.unwrap_or(format_ident!("MutationKey"));
    let key_wrapper = format_ident!("{}KeyWrapper", ident);

    let mut key_consts = Vec::new();
    match data {
        Data::Enum(data) => {
            for variant in data.variants {
                let variant_ident = variant.ident;
                let variant_string = variant_ident.to_string().to_case(Case::Camel);

                let variant_ty = match variant.fields {
                    Fields::Named(_) => {
                        panic!("The 'Mutation' derive macros does not support named fields.")
                    }
                    Fields::Unnamed(fields) => match fields.unnamed.len() {
                        0 => quote! { () },
                        1 => {
                            let field_ty = fields.unnamed[0].ty.clone();
                            quote! { #field_ty}
                        }
                        _ => {
                            panic!("The 'Mutation' derive macro requires all enum variants to have at most one unnamed field.");
                        }
                    },
                    Fields::Unit => quote! { () },
                };
                key_consts.push(quote! { const #variant_ident: #key_wrapper<#variant_ty> = #key_wrapper(#variant_string, std::marker::PhantomData); });
            }
        }
        _ => panic!("The 'Key' derive macro is only supported on enums!"),
    }

    quote! {
        impl #crate_name::KeyDefinition for #ident {
            type Key = #key_ident;
        }

        #[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
        pub struct #key_ident;

        pub struct #key_wrapper<TArg>(&'static str, std::marker::PhantomData<TArg>);

        impl<TArg> #crate_name::Key<#key_ident, TArg> for #key_wrapper<TArg> {
            type Arg = TArg;

            fn to_val(&self) -> String {
                self.0.into()
            }
        }

        #[allow(non_upper_case_globals)]
        impl #key_ident {
            #(#key_consts)*
        }
    }
    .into()
}
