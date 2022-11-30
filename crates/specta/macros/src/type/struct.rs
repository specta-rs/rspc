use crate::utils::unraw_raw_ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, Fields, GenericParam, Generics};

use super::{attr::*, generics::construct_datatype};

pub fn parse_struct(
    struct_name: &str,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> (TokenStream, TokenStream, bool) {
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
        quote! {
            generics.get(#i).cloned().unwrap_or_else(||
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

    let definition_generics = generic_idents
        .iter()
        .map(|(_, ident)| quote!(stringify!(#ident)));

    let definition = match &data.fields {
        Fields::Named(_) => {
            let fields = data.fields.iter().filter_map(|field| {
                let field_attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

                if field_attrs.skip {
                    return None;
                }

                let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

                let ty = construct_datatype(
                    format_ident!("ty"),
                    field_ty,
                    &generic_idents,
                    crate_ref,
                    field_attrs.inline,
                );

                let field_ident_str = unraw_raw_ident(field.ident.as_ref().unwrap());

                let field_name = match (field_attrs.rename, container_attrs.rename_all) {
                    (Some(name), _) => name,
                    (_, Some(inflection)) => inflection.apply(&field_ident_str),
                    (_, _) => field_ident_str,
                };

                let optional = field_attrs.optional;
                let flatten = field_attrs.flatten;

                let ty = if field_attrs.flatten {
                    quote! {
                        #[allow(warnings)]
                        {
                            #ty
                        }

                        fn validate_flatten<T: #crate_ref::Flatten>() {}
                        validate_flatten::<#field_ty>();

                        let mut ty = <#field_ty as #crate_ref::Type>::inline(#crate_ref::DefOpts {
                            parent_inline: false,
                            type_map: opts.type_map
                        }, &generics);

                        match &mut ty {
                            #crate_ref::DataType::Enum(e) => {
                                e.make_flattenable();
                            }
                            _ => {}
                        }

                        ty
                    }
                } else {
                    quote! {
                        #ty

                        ty
                    }
                };

                Some(quote!(#crate_ref::ObjectField {
                    name: #field_name.to_string(),
                    optional: #optional,
                    flatten: #flatten,
                    ty: {
                        #ty
                    }
                }))
            });

            let tag = container_attrs
                .tag
                .as_ref()
                .map(|t| quote!(Some(#t.to_string())))
                .unwrap_or(quote!(None));

            quote!(#crate_ref::ObjectType {
                name: #struct_name.to_string(),
                generics: vec![#(#definition_generics),*],
                fields: vec![#(#fields),*],
                tag: #tag,
                type_id: Some(std::any::TypeId::of::<Self>())
            }.into())
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

            quote!(#crate_ref::TupleType {
                name: #struct_name.to_string(),
                generics: vec![#(#definition_generics),*],
                fields: vec![#(#fields),*]
            }.into())
        }
        Fields::Unit => {
            quote!(#crate_ref::TupleType {
                name: #struct_name.to_string(),
                generics: vec![#(#definition_generics),*],
                fields: vec![],
            }.into())
        }
    };

    let category = quote! {
        #crate_ref::TypeCategory::Reference {
            reference: #crate_ref::DataType::Reference {
                name: #struct_name.to_string(),
                generics: vec![#(#reference_generics),*],
                type_id: std::any::TypeId::of::<Self>()
            },
            // TODO: make accurate
            placeholder: #crate_ref::ObjectType {
                name: #struct_name.to_string(),
                generics: vec![],
                fields: vec![],
                tag: None,
                type_id: Some(std::any::TypeId::of::<Self>())
            }.into()
        }
    };

    (definition, category, true)
}
