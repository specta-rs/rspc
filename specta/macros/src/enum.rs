use crate::{
    attr::{ContainerAttr, EnumAttr, FieldAttr, Tagged, VariantAttr},
    construct_datatype,
    utils::unraw_raw_ident,
};
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
) -> (TokenStream, TokenStream) {
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

            let variant_name_str = match (attrs.rename.clone(), container_attrs.rename_all) {
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

                        let field_name = field_attrs
                            .rename
                            .clone()
                            .unwrap_or(unraw_raw_ident(field.ident.as_ref().unwrap()));

                        quote!(#crate_ref::ObjectField {
                            name: #field_name.to_string(),
                            optional: false,
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
                        tag: None
                    }))
                }
            }
        });

    let repr = match enum_attrs
        .tagged()
        .expect("Invalid tag/content combination")
    {
        Tagged::Externally => quote!(External),
        Tagged::Untagged => quote!(Untagged),
        Tagged::Adjacently { tag, content } => {
            quote!(Adjacent { tag: #tag.to_string(), content: #content.to_string() })
        }
        Tagged::Internally { tag } => {
            quote!(Internal { tag: #tag.to_string() })
        }
    };

    (
        quote!(#crate_ref::DataType::Enum(#crate_ref::EnumType {
            name: #enum_name_str.to_string(),
            generics: vec![#(#definition_generics),*],
            variants: vec![#(#variants),*],
            repr: #crate_ref::EnumRepr::#repr
        })),
        quote!(#crate_ref::DataType::Reference {
            name: #enum_name_str.to_string(),
            generics: vec![#(#reference_generics),*],
        }),
    )
}
