#[macro_use]
mod utils;
mod attr;
mod r#enum;
mod generics;
mod r#struct;

use r#enum::parse_enum;
use r#struct::parse_struct;

use attr::{ContainerAttr, EnumAttr};
use generics::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Type, attributes(specta, serde))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_type_internal(input, "specta".into())
}

/// This macro is exposed from rspc as a wrapper around [Type] with a correct import path.
/// This is exposed from here so rspc doesn't need a macro package for 4 lines of code.
#[doc(hidden)]
#[proc_macro_derive(RSPCType, attributes(specta, serde))]
pub fn derive_rspc_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_type_internal(input, "rspc::internal::specta".into())
}

fn derive_type_internal(
    input: proc_macro::TokenStream,
    default_crate_name: String,
) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input);

    let DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = &derive_input;

    let container_attrs = ContainerAttr::from_attrs(attrs).unwrap();

    let crate_name: TokenStream = container_attrs
        .crate_name
        .clone()
        .unwrap_or_else(|| default_crate_name)
        .parse()
        .unwrap();
    let crate_ref = quote!(::#crate_name);

    let name_str = container_attrs.rename.clone().unwrap_or(ident.to_string());

    let (inlines, reference) = match data {
        Data::Struct(data) => {
            parse_struct(&name_str, &container_attrs, &generics, &crate_ref, data)
        }
        Data::Enum(data) => {
            let enum_attrs = EnumAttr::from_attrs(attrs).unwrap();

            parse_enum(
                &name_str,
                &enum_attrs,
                &container_attrs,
                &generics,
                &crate_ref,
                data,
            )
        }
        _ => panic!("Type 'Union' is not supported by specta!"),
    };

    let definition_generics = generics.type_params().map(|param| {
        let ident = &param.ident;

        quote!(#crate_ref::DataType::Generic(stringify!(#ident).to_string()))
    });

    let flatten_impl = match data {
        Data::Struct(_) => {
            let heading = impl_heading(quote!(#crate_ref::Flatten), &ident, &generics);

            Some(quote! {
                #heading {}
            })
        }
        _ => None,
    };

    let type_impl_heading = impl_heading(quote!(#crate_ref::Type), &ident, &generics);

    let out = quote! {
        #type_impl_heading {
            const NAME: &'static str = #name_str;

            fn inline(opts: #crate_ref::DefOpts, generics: &[#crate_ref::DataType]) -> #crate_ref::DataType {
                #inlines
            }

            fn reference(opts: #crate_ref::DefOpts, generics: &[#crate_ref::DataType]) -> #crate_ref::DataType {
                if !opts.type_map.contains_key(&Self::NAME) {
                    Self::definition(#crate_ref::DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map
                    });
                }

                #reference
            }

            fn definition(opts: #crate_ref::DefOpts) -> #crate_ref::DataType {
                if !opts.type_map.contains_key(Self::NAME) {
                    opts.type_map.insert(Self::NAME, #crate_ref::DataType::Object(#crate_ref::ObjectType {
                        name: #name_str.to_string(),
                        generics: vec![],
                        fields: vec![],
                        tag: None
                    }));

                    let def = Self::inline(#crate_ref::DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map
                    }, &[#(#definition_generics),*]);

                    opts.type_map.insert(Self::NAME, def.clone());
                }

                opts.type_map.get(Self::NAME).unwrap().clone()
            }
        }

        #flatten_impl
    };

    println!("{}", out.to_string());

    out.into()
}
