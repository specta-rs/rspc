use super::{attr::*, generics::construct_datatype};
use crate::utils::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Fields, GenericParam, Generics};

pub fn parse_enum(
    enum_name_str: &str,
    enum_attrs: &EnumAttr,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataEnum,
) -> (TokenStream, TokenStream, bool) {
    let generic_idents = generics
        .params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            GenericParam::Type(t) => Some((i, &t.ident)),
            _ => None,
        });

    let definition_generics = generic_idents.clone().map(|(_, ident)| {
        let ident = &ident.clone();

        quote!(stringify!(#ident))
    });

    let reference_generics = generic_idents.clone().map(|(i, ident)| {
        let ident = &ident.clone();

        quote! {
            generics.get(#i).cloned().unwrap_or_else(
                || <#ident as #crate_ref::Type>::reference(
                    #crate_ref::DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map
                    },
                    &[]
                )
            )
        }
    });

    let repr = enum_attrs
        .tagged()
        .expect("Invalid tag/content combination");

    let (repr_tokens, can_flatten) = match repr {
        Tagged::Externally => (
            quote!(External),
            data.variants.iter().any(|v| match &v.fields {
                Fields::Unnamed(f) if f.unnamed.len() == 1 => true,
                Fields::Named(_) => true,
                _ => false,
            }),
        ),
        Tagged::Untagged => (
            quote!(Untagged),
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
        Tagged::Adjacently { tag, content } => (
            quote!(Adjacent { tag: #tag.to_string(), content: #content.to_string() }),
            true,
        ),
        Tagged::Internally { tag } => (
            quote!(Internal { tag: #tag.to_string() }),
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
    };

    let variants = data
        .variants
        .iter()
        .map(|v| {
            let attrs = VariantAttr::from_attrs(&v.attrs).expect("Failed to parse enum attributes");

            (v, attrs)
        })
        .filter(|(_, attrs)| !attrs.skip)
        .map(|(variant, attrs)| {
            let variant_ident_str = unraw_raw_ident(&variant.ident);

            let variant_name_str = match (attrs.rename, container_attrs.rename_all) {
                (Some(name), _) => name,
                (_, Some(inflection)) => inflection.apply(&variant_ident_str),
                (_, _) => variant_ident_str,
            };

            let generic_idents = generic_idents.clone().collect::<Vec<_>>();

            match &variant.fields {
                Fields::Unit => {
                    quote!(#crate_ref::EnumVariant::Unit(#variant_name_str.to_string()))
                }
                Fields::Unnamed(fields) => {
                    let fields = fields.unnamed.iter().map(|field| {
                        let field_ty = &field.ty;

                        let generic_vars = construct_datatype(
                            format_ident!("gen"),
                            field_ty,
                            &generic_idents,
                            crate_ref,
                            false,
                        );

                        quote!({
                            #generic_vars

                            gen
                        })
                    });

                    quote!(#crate_ref::EnumVariant::Unnamed(#crate_ref::TupleType {
                        name: #variant_name_str.to_string(),
                        fields: vec![#(#fields),*],
                        generics: vec![]
                    }))
                }
                Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|field| {
                        let field_attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

                        let generic_vars = construct_datatype(
                            format_ident!("gen"),
                            &field.ty,
                            &generic_idents,
                            crate_ref,
                            false,
                        );

                        let field_ident_str = unraw_raw_ident(field.ident.as_ref().unwrap());

                        let field_name = match (field_attrs.rename, attrs.rename_all) {
                            (Some(name), _) => name,
                            (_, Some(inflection)) => inflection.apply(&field_ident_str),
                            (_, _) => field_ident_str,
                        };

                        quote!(#crate_ref::ObjectField {
                            name: #field_name.to_string(),
                            optional: false,
                            flatten: false,
                            ty: {
                                #generic_vars

                                gen
                            },
                        })
                    });

                    quote!(#crate_ref::EnumVariant::Named(#crate_ref::ObjectType {
                        name: #variant_name_str.to_string(),
                        fields: vec![#(#fields),*],
                        generics: vec![],
                        tag: None,
                        type_id: None
                    }))
                }
            }
        });

    (
        quote!(#crate_ref::EnumType {
            name: #enum_name_str.to_string(),
            generics: vec![#(#definition_generics),*],
            variants: vec![#(#variants),*],
            repr: #crate_ref::EnumRepr::#repr_tokens,
            type_id: std::any::TypeId::of::<Self>()
        }.into()),
        quote! {
            #crate_ref::TypeCategory::Reference {
                reference: #crate_ref::DataType::Reference {
                    name: #enum_name_str.to_string(),
                    generics: vec![#(#reference_generics),*],
                    type_id: std::any::TypeId::of::<Self>()
                },
                // TODO: make accurate
                placeholder: #crate_ref::ObjectType {
                    name: #enum_name_str.to_string(),
                    generics: vec![],
                    fields: vec![],
                    tag: None,
                    type_id: Some(std::any::TypeId::of::<Self>())
                }.into()
            }
        },
        can_flatten,
    )
}
