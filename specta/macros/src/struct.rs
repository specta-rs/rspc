use crate::{
    attr::{ContainerAttr, FieldAttr},
    utils::unraw_raw_ident,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, Fields, GenericParam, Generics};

use crate::construct_datatype;

pub fn parse_struct(
    struct_name: &str,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> (TokenStream, TokenStream) {
    let generic_idents = generics
        .params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            GenericParam::Type(t) => Some((i, &t.ident)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let reference_generics = generic_idents.iter().map(|(i, ident)| {
        let ident = &ident.clone();

        quote! {
            generics.get(#i).cloned().unwrap_or(
                <#ident as #crate_ref::Type>::reference(
                    #crate_ref::DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map
                    },
                    &[]
                )
            )
        }
    });

    let definition_generics = generic_idents.iter().map(|(_, ident)| {
        let ident = &ident.clone();

        quote!(stringify!(#ident))
    });

    let definition = match &data.fields {
        Fields::Named(_) => {
            let fields = data.fields.iter().filter_map(|field| {
                let field_attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

                if field_attrs.skip {
                    return None;
                }

                let ty = construct_datatype(
                    format_ident!("ty"),
                    &field.ty,
                    &generic_idents,
                    crate_ref,
                    field_attrs.inline,
                );

                let field_ident_str = unraw_raw_ident(field.ident.as_ref().unwrap());

                let field_name = match (field_attrs.rename, container_attrs.rename_all) {
                    (Some(name), _) => name,
                    (_, Some(inflection)) => inflection.apply(&field_ident_str),
                    (_, _) => field_ident_str.to_string(),
                };

                let optional = field_attrs.optional;


                if field_attrs.flatten {
                    let field_ty = &field.ty;

                    Some(quote! {{
                        #[allow(warnings)]
                        {
                            #ty
                        }

                        <#field_ty as #crate_ref::Flatten>::flatten(#crate_ref::DefOpts {
                            parent_inline: false,
                            type_map: opts.type_map
                        }, &generics)
                    }})
                } else {
                    Some(quote!(vec![#crate_ref::ObjectField {
                        name: #field_name.to_string(),
                        optional: #optional,
                        ty: {
                            #ty

                            ty
                        },
                    }]))
                }
            });

            let tag = container_attrs
                .tag
                .as_ref()
                .map(|t| quote!(Some(#t.to_string())))
                .unwrap_or(quote!(None));

            quote!(#crate_ref::DataType::Object(#crate_ref::ObjectType {
                name: #struct_name.to_string(),
                generics: vec![#(#definition_generics),*],
                fields: (vec![#(#fields),*] as Vec<Vec<#crate_ref::ObjectField>>)
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                tag: #tag
            }))
        }
        Fields::Unnamed(_) => {
            let fields = data.fields.iter().filter_map(|field| {
                let field_attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

                if field_attrs.skip {
                    return None;
                }

                let generic_vars = construct_datatype(
                    format_ident!("gen"),
                    &field.ty,
                    &generic_idents,
                    crate_ref,
                    field_attrs.inline,
                );

                Some(quote! {{
                    #generic_vars

                    gen
                }})
            });

            quote!(#crate_ref::DataType::Tuple(#crate_ref::TupleType {
                name: #struct_name.to_string(),
                generics: vec![#(#definition_generics),*],
                fields: vec![#(#fields),*]
            }))
        }
        Fields::Unit => quote!(#crate_ref::DataType::Tuple(#crate_ref::TupleType {
            name: #struct_name.to_string(),
            generics: vec![#(#definition_generics),*],
            fields: vec![],
        })),
    };

    let reference = quote!(#crate_ref::DataType::Reference {
        name: #struct_name.to_string(),
        generics: vec![#(#reference_generics),*],
    });

    (definition, reference)
}
