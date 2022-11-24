use attr::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use r#enum::parse_enum;
use r#struct::parse_struct;
use syn::{parse_macro_input, Data, DeriveInput};

use generics::impl_heading;

mod attr;
mod r#enum;
mod generics;
mod r#struct;

pub fn derive(
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
        .unwrap_or(default_crate_name)
        .parse()
        .unwrap();
    let crate_ref = quote!(::#crate_name);

    let name_str = container_attrs
        .rename
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let (inlines, category, can_flatten) = match data {
        Data::Struct(data) => parse_struct(&name_str, &container_attrs, generics, &crate_ref, data),
        Data::Enum(data) => {
            let enum_attrs = EnumAttr::from_attrs(attrs).unwrap();

            parse_enum(
                &name_str,
                &enum_attrs,
                &container_attrs,
                generics,
                &crate_ref,
                data,
            )
        }
        _ => panic!("Type 'Union' is not supported by specta!"),
    };

    let definition_generics = generics.type_params().map(|param| {
        let ident = &param.ident;

        quote!(#crate_ref::datatype::GenericType(stringify!(#ident).to_string()))
    });

    let flatten_impl = can_flatten.then(|| {
        let heading = impl_heading(quote!(#crate_ref::Flatten), ident, generics);
        quote!(#heading {})
    });

    let type_impl_heading = impl_heading(quote!(#crate_ref::Type), ident, generics);

    let export = {
        let export_fn_name = format_ident!("__push_specta_type_{}", ident);

        let generic_params = generics
            .params
            .iter()
            .filter(|param| matches!(param, syn::GenericParam::Type(_)))
            .map(|_| quote! { () });
        let ty = quote!(<#ident<#(#generic_params),*> as #crate_ref::Type>);

        quote! {
            #[#crate_ref::ctor::ctor]
            #[allow(non_snake_case)]
            fn #export_fn_name() {
                let type_map = &mut *#crate_ref::export::TYPES.lock().unwrap();

                #ty::reference(
                    #crate_ref::r#type::DefOpts {
                        parent_inline: false,
                        type_map
                    },
                    &[]
                );
            }
        }
    };

    quote! {
        #type_impl_heading {
            const NAME: &'static str = #name_str;

            fn inline(opts: #crate_ref::r#type::DefOpts, generics: &[#crate_ref::datatype::DataType]) -> #crate_ref::datatype::DataType {
                #inlines
            }

            fn category_impl(opts: #crate_ref::r#type::DefOpts, generics: &[#crate_ref::datatype::DataType]) -> #crate_ref::TypeCategory {
                #category
            }

            fn definition_generics() -> Vec<#crate_ref::datatype::GenericType> {
                vec![#(#definition_generics),*]
            }
        }

        #export

        #flatten_impl
    }.into()
}
