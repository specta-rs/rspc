#[macro_use]
mod utils;
mod attr;
mod generics;

use attr::{ContainerAttr, EnumAttr, FieldAttr, Tagged, VariantAttr};
use generics::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, GenericParam,
    Generics, Ident, Type,
};

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

    let name = match container_attrs.inline {
        true => quote!(None),
        false => quote!(Some(#name_str.to_string())),
    };

    let ty = match data {
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

    let impl_for_type = generate_impl(&crate_name, &ident, &generics);

    quote! {
       #impl_for_type {
            fn def(defs: &mut #crate_ref::TypeDefs) -> #crate_ref::DataType {
                #ty
            }

            fn name() -> Option<String> {
                #name
            }
        }
    }
    .into()
}

fn parse_struct(
    struct_name: &str,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> TokenStream {
    let fields = data
        .fields
        .iter()
        .map(|field| {
            let attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

            (field, attrs)
        })
        .filter(|(_, attrs)| !attrs.skip);

    let generic_types = generics.type_params().map(|p| {
        let ident = &p.ident;
        quote!(#crate_ref::GenericType {
            name: stringify!(#ident).to_string(),
        })
    });

    let (ty, fields, add_fields_arms) = match &data.fields {
        Fields::Unit => {
            let ty = quote! {
                #crate_ref::DataType::Tuple(#crate_ref::TupleType {
                    name: #struct_name.to_string(),
                    fields: vec![],
                    generics: vec![],
                })
            };

            (ty, quote!(vec![] as Vec<()>), quote!(_ => {}))
        }
        Fields::Unnamed(_) => {
            let fields = fields.clone().map(|(field, field_attrs)| {
                parse_tuple_struct_field(
                    &field,
                    &generics,
                    &container_attrs,
                    &field_attrs,
                    crate_ref,
                )
            });

            let fields = quote! {
                (vec![#(#fields),*] as Vec<Vec<_>>)
                    .into_iter()
                    .flatten()
                    .collect::<Vec<#crate_ref::DataType>>()
            };

            let ty = quote! {
                #crate_ref::DataType::Tuple(#crate_ref::TupleType {
                    name: #struct_name.to_string(),
                    fields: vec![],
                    generics: vec![#(#generic_types),*],
                })
            };

            let add_fields = quote! {
                #crate_ref::DataType::Tuple(#crate_ref::TupleType {
                    fields,
                    ..
                }) => fields.extend(new_fields),
                _ => {
                    println!("A");
                    unreachable!()
                },
            };

            (ty, fields, add_fields)
        }
        Fields::Named(_) => {
            let fields = fields.clone().map(|(field, field_attrs)| {
                parse_named_struct_field(
                    field,
                    &generics,
                    &container_attrs,
                    &field_attrs,
                    crate_ref,
                )
            });

            let fields = quote! {
                (vec![#(#fields),*] as Vec<Vec<_>>)
                    .into_iter()
                    .flatten()
                    .collect::<Vec<#crate_ref::ObjectField>>()
            };

            let ty = quote! {
                #crate_ref::DataType::Object(#crate_ref::ObjectType {
                    name: #struct_name.to_string(),
                    inline: false,
                    fields: vec![],
                    generics: vec![#(#generic_types),*],
                    tag: None
                })
            };

            let add_fields = quote! {
                #crate_ref::DataType::Object(#crate_ref::ObjectType {
                    fields,
                    ..
                }) => fields.extend(new_fields),
                v => {
                    println!("B {:?}", v);
                    // unreachable!()
                },
            };

            (ty, fields, add_fields)
        }
    };

    let generic_refs = generic_refs(generics, crate_ref);

    match container_attrs.inline {
        // If the struct is not inline, retrieve the struct definition and
        // add fields to it. The struct's fields definitions are constructed
        // after the definition is inserted into the export map so that they
        // are aware of structs that have been defined earlier.
        false => quote! {{
            if !defs.contains_key(#struct_name) {
                defs.insert(
                    #struct_name.to_string(),
                    #ty
                );
            }

            let new_fields = #fields;

            match defs.get_mut(#struct_name).unwrap() {
                #add_fields_arms
            }

            #crate_ref::DataType::Reference {
                name: #struct_name.to_string(),
                generics: #generic_refs
            }
        }},
        // If the struct is inline, we return the struct definition
        true => quote! {{
            let ty = #ty;

            let new_fields = #fields;

            match &mut ty {
                #add_fields_arms
            }

            ty
        }},
    }
}

fn parse_named_struct_field(
    field: &Field,
    generics: &Generics,
    container_attrs: &ContainerAttr,
    field_attrs: &FieldAttr,
    crate_ref: &TokenStream,
) -> TokenStream {
    let ty_ident = &field.ty;

    let name_str = sanitise_raw_ident(field.ident.as_ref().unwrap());

    let name_str = match (field_attrs.rename.clone(), container_attrs.rename_all) {
        (Some(name), _) => name,
        (None, Some(inflection)) => inflection.apply(&name_str),
        (None, None) => name_str,
    };

    let optional = field_attrs.optional;

    let ty = match generics.type_params().any(|p| {
        let g_ident = &p.ident;

        quote!(#g_ident).to_string() == quote!(#ty_ident).to_string()
    }) {
        true => {
            quote!(#crate_ref::DataType::Generic {
                ident: stringify!(#ty_ident).to_string(),
            })
        }
        false => {
            // TODO: flatten + optional
            // if field_attrs.flatten {
            //     return quote! {{
            //         let ty = #crate_ref::upsert_datatype::<#ty_ident>(defs);
            //
            //         match ty {
            //             #crate_ref::DataType::Object(#crate_ref::ObjectType { fields, .. }) => fields,
            //             _ => panic!("Attempted to flatten non-object field")
            //         }
            //     }};
            // }

            match field_attrs.inline {
                true => quote! {
                    match <#ty_ident as #crate_ref::Type>::name() {
                        Some(name) => {
                            match defs.get(&name) {
                                Some(def) => def.clone(),
                                None => <#ty_ident as #crate_ref::Type>::def(defs),
                            }
                        }
                        None => {
                            <#ty_ident as #crate_ref::Type>::def(defs)
                            // panic!("Attempted to inline non-inline type"),
                        }
                    }
                },
                false => quote!(<#ty_ident as #crate_ref::Type>::def(defs)),
            }
        }
    };

    quote! {{
        vec![#crate_ref::ObjectField {
            name: #name_str.to_string(),
            optional: #optional,
            ty: #ty,
        }]
    }}
}

fn parse_tuple_struct_field(
    field: &Field,
    generics: &Generics,
    _container_attrs: &ContainerAttr,
    field_attrs: &FieldAttr,
    crate_ref: &TokenStream,
) -> TokenStream {
    let ty_ident = &field.ty;

    // // TODO: flatten + optional
    // if field_attrs.flatten {
    //     return quote! {{
    //         match ty {
    //             #crate_ref::DataType::Object(ObjectType { fields, .. }) => fields
    //                 .into_iter()
    //                 .map(|of| of.ty)
    //                 .collect::<Vec<#crate_ref::DataType>>(),
    //             _ => panic!("Attempted to flatten non-object field"),
    //         }
    //     }};
    // }

    // let optional = field_attrs.optional.then(|| {
    //     quote! {
    //         ty = #crate_ref::DataType::Nullable(Box::new(def.clone()))
    //     }
    // });

    let ty = match generics.type_params().any(|p| {
        let g_ident = &p.ident;

        quote!(#g_ident).to_string() == quote!(#ty_ident).to_string()
    }) {
        true => {
            quote!(#crate_ref::DataType::Generic {
                ident: stringify!(#ty_ident).to_string(),
            })
        }
        false => {
            let ty = match field_attrs.inline {
                true => quote! {
                    match defs.get(<#ty_ident as #crate_ref::Type>::name()) {
                        Some(def) => def.clone(),
                        None => <#ty_ident as #crate_ref::Type>::def(defs),
                    }
                },
                false => quote!(<#ty_ident as #crate_ref::Type>::def(defs)),
            };

            quote! {
                let ty = #ty;

                // #optional;

                ty
            }
        }
    };

    quote! {{
        let ty = {
            #ty
        };

        vec![ty]
    }}
}

fn parse_enum(
    enum_name_str: &str,
    enum_attrs: &EnumAttr,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataEnum,
) -> TokenStream {
    if data.variants.len() == 0 {
        return quote!(#crate_ref::DataType::Primitive(#crate_ref::PrimitiveType::Never));
    }

    let variants = data
        .variants
        .iter()
        .map(|v| {
            let attrs = VariantAttr::from_attrs(&v.attrs).expect("Failed to parse enum attributes");

            (v, attrs)
        })
        .filter(|(_, attrs)| !attrs.skip)
        .map(|(variant, attrs)| {
            let variant_name_str = variant.ident.to_string();

            let variant_name_str = match (attrs.rename.clone(), container_attrs.rename_all) {
                (Some(name), _) => name,
                (None, Some(inflection)) => inflection.apply(&variant_name_str),
                (None, None) => variant_name_str,
            };

            match &variant.fields {
                Fields::Unit => {
                    quote!(#crate_ref::EnumVariant::Unit(#variant_name_str.to_string()))
                }
                Fields::Unnamed(_) => {
                    let fields = variant.fields.iter().map(|field| {
                        let ty_ident = &field.ty;

                        match generics.type_params().any(|p| {
                            let g_ident = &p.ident;

                            quote!(#g_ident).to_string() == quote!(#ty_ident).to_string()
                        }) {
                            true => quote!(#crate_ref::DataType::Generic {
                                ident: stringify!(#ty_ident).to_string(),
                            }),
                            false => quote!(<#ty_ident as #crate_ref::Type>::def(defs)),
                        }
                    });

                    quote!(#crate_ref::EnumVariant::Unnamed(#crate_ref::TupleType {
                        name: #variant_name_str.to_string(),
                        fields: vec![#(#fields),*],
                        generics: vec![]
                    }))
                }
                Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|f| {
                        let ident = f.ident.as_ref().expect("Named field has no ident");

                        let name = sanitise_raw_ident(ident);

                        let ty = &f.ty;

                        let ty = match generics.type_params().any(|p| {
                            let g_ident = &p.ident;

                            quote!(#g_ident).to_string() == quote!(#ty).to_string()
                        }) {
                            true => quote!(#crate_ref::DataType::Generic {
                                ident: stringify!(#ty).to_string(),
                            }),
                            false => quote!(<#ty as #crate_ref::Type>::def(defs)),
                        };

                        quote!(#crate_ref::ObjectField {
                            name: #name.into(),
                            ty: #ty,
                            optional: false,
                        })
                    });

                    quote!(#crate_ref::EnumVariant::Named(#crate_ref::ObjectType {
                        name: #variant_name_str.to_string(),
                        inline: true,
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

    // Generics identifiers on the enum definition
    let generic_types = generics.type_params().map(|p| {
        let ident = &p.ident;
        quote!(#crate_ref::GenericType {
            name: stringify!(#ident).to_string(),
        })
    });

    // Generic types to be provided to usages of the enum
    let generic_refs = generic_refs(generics, crate_ref);

    let ty = quote!(#crate_ref::DataType::Enum(#crate_ref::EnumType {
        name: #enum_name_str.to_string(),
        variants: vec![],
        generics: vec![#(#generic_types),*],
        repr: #crate_ref::EnumRepr::#repr,
    }));

    let insert_lazy_match_arms = quote! {
        #crate_ref::DataType::Enum(#crate_ref::EnumType {
            variants,
            ..
        }) => {
            variants.extend(new_variants);
        }
        _ => {
            println!("C");
            unreachable!()
        },
    };

    match container_attrs.inline {
        false => quote! {{
            if !defs.contains_key(#enum_name_str) {
                defs.insert(
                    #enum_name_str.to_string(),
                    #ty
                );
            }

            let new_variants = vec![#(#variants),*];

            match defs.get_mut(#enum_name_str).unwrap() {
                #insert_lazy_match_arms
            }

            #crate_ref::DataType::Reference {
                name: #enum_name_str.to_string(),
                generics: #generic_refs
            }
        }},
        true => quote! {{
            let ty = #ty;

            let new_variants = vec![#(#variants),*];

            // match &mut ty {
            //     #insert_lazy_match_arms
            // }

            ty
        }},
    }
}

fn sanitise_raw_ident(ident: &Ident) -> String {
    let ident = ident.to_string();
    if ident.starts_with("r#") {
        ident.trim_start_matches("r#").to_owned()
    } else {
        ident
    }
}
