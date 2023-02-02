use crate::utils::{pass_attrs, unraw_raw_ident};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, Field, Fields, GenericParam, Generics};

use super::{attr::*, generics::construct_datatype};

pub fn decode_field_attrs(field: &Field) -> syn::Result<(&Field, FieldAttr)> {
    // We pass all the attributes at the start and when decoding them pop them off the list.
    // This means at the end we can check for any that weren't consumed and throw an error.
    let mut attrs = pass_attrs(&field.attrs)?;
    let field_attrs = FieldAttr::from_attrs(&mut attrs)?;

    for attr in attrs
        .into_iter()
        .filter(|attr| attr.root_ident() == "specta")
    {
        return Err(syn::Error::new(
            attr.key_span(),
            format!("specta: Found unsupported field attribute '{}'", attr.tag()),
        ));
    }

    Ok((field, field_attrs))
}

pub fn parse_struct(
    (container_attrs, struct_attrs): (&ContainerAttr, StructAttr),
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> syn::Result<(TokenStream, TokenStream, bool)> {
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
            let fields = data.fields.iter().map(decode_field_attrs)
            .collect::<syn::Result<Vec<_>>>()?
            .iter()
            .filter_map(|(field, field_attrs)| {

                if field_attrs.skip {
                    return None;
                }

                Some((field, field_attrs))
            }).map(|(field, field_attrs)| {
                let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

                let ty = construct_datatype(
                    format_ident!("ty"),
                    field_ty,
                    &generic_idents,
                    crate_ref,
                    field_attrs.inline,
                )?;

                let field_ident_str = unraw_raw_ident(field.ident.as_ref().unwrap());

                let field_name = match (field_attrs.rename.clone(), container_attrs.rename_all) {
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

                Ok(quote!(#crate_ref::ObjectField {
                    name: #field_name,
                    optional: #optional,
                    flatten: #flatten,
                    ty: {
                        #ty
                    }
                }))
            }).collect::<syn::Result<Vec<TokenStream>>>()?;

            let tag = container_attrs
                .tag
                .as_ref()
                .map(|t| quote!(Some(#t)))
                .unwrap_or(quote!(None));

            quote!(#crate_ref::ObjectType {
                name: <Self as #crate_ref::Type>::NAME,
                generics: vec![#(#definition_generics),*],
                fields: vec![#(#fields),*],
                tag: #tag,
                type_id: Some(std::any::TypeId::of::<Self>())
            }.into())
        }
        Fields::Unnamed(_) => {
            if struct_attrs.transparent {
                let ty = &data.fields.iter().next().unwrap().ty;
                quote!(#ty)
            } else {
                let fields = data
                    .fields
                    .iter()
                    .map(decode_field_attrs)
                    .collect::<syn::Result<Vec<_>>>()?
                    .iter()
                    .filter_map(|(field, field_attrs)| {
                        if field_attrs.skip {
                            return None;
                        }

                        Some((field, field_attrs))
                    })
                    .map(|(field, field_attrs)| {
                        let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

                        let generic_vars = construct_datatype(
                            format_ident!("gen"),
                            field_ty,
                            &generic_idents,
                            crate_ref,
                            field_attrs.inline,
                        )?;

                        Ok(quote! {{
                            #generic_vars

                            gen
                        }})
                    })
                    .collect::<syn::Result<Vec<TokenStream>>>()?;

                quote!(#crate_ref::TupleType {
                    name: <Self as #crate_ref::Type>::NAME,
                    generics: vec![#(#definition_generics),*],
                    fields: vec![#(#fields),*]
                }.into())
            }
        }
        Fields::Unit => {
            quote!(#crate_ref::TupleType {
                name: <Self as #crate_ref::Type>::NAME,
                generics: vec![#(#definition_generics),*],
                fields: vec![],
            }.into())
        }
    };

    let category = if container_attrs.inline {
        quote!(#crate_ref::TypeCategory::Inline({
            let generics = &[#(#reference_generics),*];
            <Self as #crate_ref::Type>::inline(opts, generics)
        }))
    } else {
        quote! {
            #crate_ref::TypeCategory::Reference {
                reference: #crate_ref::DataType::Reference {
                    name: <Self as #crate_ref::Type>::NAME,
                    generics: vec![#(#reference_generics),*],
                    type_id: std::any::TypeId::of::<Self>()
                },
                placeholder: #crate_ref::DataType::Placeholder,
            }
        }
    };

    Ok((definition, category, true))
}
