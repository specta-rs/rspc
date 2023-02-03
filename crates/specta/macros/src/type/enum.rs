use super::{attr::*, generics::construct_datatype, r#struct::decode_field_attrs};
use crate::utils::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Fields, GenericParam, Generics};

pub fn parse_enum(
    enum_attrs: &EnumAttr,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataEnum,
) -> syn::Result<(TokenStream, TokenStream, bool)> {
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

    let repr = enum_attrs.tagged()?;

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
        Tagged::Adjacently { tag, content } => {
            (quote!(Adjacent { tag: #tag, content: #content }), true)
        }
        Tagged::Internally { tag } => (
            quote!(Internal { tag: #tag }),
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
    };

    let variants = data
        .variants
        .iter()
        .map(|v| {
            // We pass all the attributes at the start and when decoding them pop them off the list.
            // This means at the end we can check for any that weren't consumed and throw an error.
            let mut attrs = pass_attrs(&v.attrs)?;
            let variant_attrs = VariantAttr::from_attrs(&mut attrs)?;

            for attr in attrs
                .into_iter()
                .filter(|attr| attr.root_ident() == "specta")
            {
                return Err(syn::Error::new(
                    attr.key_span(),
                    format!(
                        "specta: Found unsupported variant attribute '{}'",
                        attr.tag()
                    ),
                ));
            }

            Ok((v, variant_attrs))
        })
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, attrs)| !attrs.skip)
        .map(|(variant, attrs)| {
            let variant_ident_str = unraw_raw_ident(&variant.ident);

            let variant_name_str = match (attrs.rename, container_attrs.rename_all) {
                (Some(name), _) => name,
                (_, Some(inflection)) => inflection.apply(&variant_ident_str),
                (_, _) => variant_ident_str,
            };

            let generic_idents = generic_idents.clone().collect::<Vec<_>>();

            Ok(match &variant.fields {
                Fields::Unit => {
                    quote!(#crate_ref::EnumVariant::Unit(#variant_name_str))
                }
                Fields::Unnamed(fields) => {
                    let fields = fields
                        .unnamed
                        .iter()
                        .map(|field| {
                            let field_ty = &field.ty;

                            let generic_vars = construct_datatype(
                                format_ident!("gen"),
                                field_ty,
                                &generic_idents,
                                crate_ref,
                                false,
                            )?;

                            Ok(quote!({
                                #generic_vars

                                gen
                            }))
                        })
                        .collect::<syn::Result<Vec<TokenStream>>>()?;

                    quote!(#crate_ref::EnumVariant::Unnamed(#crate_ref::TupleType {
                        name: #variant_name_str,
                        fields: vec![#(#fields),*],
                        generics: vec![]
                    }))
                }
                Fields::Named(fields) => {
                    let fields = fields
                        .named
                        .iter()
                        .map(|field| {
                            let (field, field_attrs) = decode_field_attrs(field)?;

                            let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

                            let generic_vars = construct_datatype(
                                format_ident!("gen"),
                                field_ty,
                                &generic_idents,
                                crate_ref,
                                false,
                            )?;

                            let field_ident_str = unraw_raw_ident(field.ident.as_ref().unwrap());

                            let field_name = match (field_attrs.rename, attrs.rename_all) {
                                (Some(name), _) => name,
                                (_, Some(inflection)) => inflection.apply(&field_ident_str),
                                (_, _) => field_ident_str,
                            };

                            Ok(quote!(#crate_ref::ObjectField {
                                name: #field_name,
                                optional: false,
                                flatten: false,
                                ty: {
                                    #generic_vars

                                    gen
                                },
                            }))
                        })
                        .collect::<syn::Result<Vec<TokenStream>>>()?;

                    quote!(#crate_ref::EnumVariant::Named(#crate_ref::ObjectType {
                        name: #variant_name_str,
                        fields: vec![#(#fields),*],
                        generics: vec![],
                        tag: None,
                        type_id: None
                    }))
                }
            })
        })
        .collect::<syn::Result<Vec<TokenStream>>>()?;

    Ok((
        quote!(#crate_ref::EnumType {
            name: <Self as #crate_ref::Type>::NAME,
            generics: vec![#(#definition_generics),*],
            variants: vec![#(#variants),*],
            repr: #crate_ref::EnumRepr::#repr_tokens,
            type_id: std::any::TypeId::of::<Self>()
        }.into()),
        quote! {
            #crate_ref::TypeCategory::Reference {
                reference: #crate_ref::DataType::Reference {
                    name: <Self as #crate_ref::Type>::NAME,
                    generics: vec![#(#reference_generics),*],
                    type_id: std::any::TypeId::of::<Self>()
                },
                placeholder: #crate_ref::DataType::Placeholder,
            }
        },
        can_flatten,
    ))
}
