#[macro_use]
mod utils;
mod attr;

use attr::{ContainerAttr, EnumAttr, FieldAttr, StructAttr, Tagged, VariantAttr};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Ident};

#[proc_macro_derive(Type, attributes(specta, serde))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
        .unwrap_or_else(|| "specta".to_string())
        .parse()
        .unwrap();
    let crate_ref = quote!(::#crate_name);

    // TODO: Deal with struct or enum with generics
    // TODO: Struct attributes -> Renaming field, etc + Serde compatibility

    let name_str = container_attrs.rename.clone().unwrap_or(ident.to_string());

    let body = match data {
        Data::Struct(data) => {
            let struct_attrs = StructAttr::from_attrs(attrs).unwrap();

            parse_struct(&name_str, &struct_attrs, &container_attrs, &crate_ref, data)
        }
        Data::Enum(data) => {
            let enum_attrs = EnumAttr::from_attrs(attrs).unwrap();

            parse_enum(&name_str, &enum_attrs, &container_attrs, &crate_ref, data)
        }
        Data::Union(_) => panic!("Type 'Union' is not supported by specta!"),
    };

    quote! {
        impl #crate_ref::Type for #ident {
            fn def(defs: &mut #crate_ref::TypeDefs) -> #crate_ref::Typedef {
                #crate_ref::Typedef {
                    type_id: std::any::TypeId::of::<#ident>(),
                    body: #body,
                }
            }
        }
    }
    .into()
}

fn parse_struct(
    struct_name_str: &str,
    _struct_attrs: &StructAttr,
    container_attrs: &ContainerAttr,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> TokenStream {
    let inline = container_attrs.inline;

    let fields = data
        .fields
        .iter()
        .map(|field| {
            let attrs = FieldAttr::from_attrs(&field.attrs).unwrap();

            (field, attrs)
        })
        .filter(|(_, attrs)| !attrs.skip);

    match &data.fields {
        Fields::Unit => quote!(#crate_ref::DataType::Tuple(#crate_ref::TupleType {
            name: #struct_name_str.to_string(),
            inline: true,
            fields: vec![],
        })),
        Fields::Unnamed(_) => {
            let fields = fields
                .clone()
                .map(|(field, field_attrs)| {
                    parse_tuple_struct_field(&field, &container_attrs, &field_attrs, crate_ref)
                })
                .collect::<Vec<_>>();

            quote!(#crate_ref::DataType::Tuple(#crate_ref::TupleType {
                name: #struct_name_str.to_string(),
                inline: #inline,
                fields: (vec![#(#fields),*] as Vec<Vec<_>>)
                    .into_iter()
                    .flatten()
                    .collect::<Vec<#crate_ref::Typedef>>(),
            }))
        }
        Fields::Named(_) => {
            let fields = fields
                .clone()
                .map(|(field, field_attrs)| {
                    parse_named_struct_field(field, &container_attrs, &field_attrs, crate_ref)
                })
                .collect::<Vec<_>>();

            let tag = container_attrs
                .tag
                .as_ref()
                .map(|t| quote!(Some(#t.to_string())))
                .unwrap_or(quote!(None));

            quote!(#crate_ref::DataType::Object(#crate_ref::ObjectType {
                name: #struct_name_str.to_string(),
                inline: #inline,
                fields: (vec![#(#fields),*] as Vec<Vec<_>>)
                    .into_iter()
                    .flatten()
                    .collect::<Vec<#crate_ref::ObjectField>>(),
                tag: #tag
            }))
        }
    }
}

fn parse_named_struct_field(
    field: &Field,
    container_attrs: &ContainerAttr,
    field_attrs: &FieldAttr,
    crate_ref: &TokenStream,
) -> TokenStream {
    let upsert = upsert_def(field, crate_ref);

    // TODO: flatten + optional
    if field_attrs.flatten {
        return quote! {{
            let def = #upsert;

            match def.body {
                #crate_ref::DataType::Object(#crate_ref::ObjectType { fields, .. }) => fields,
                _ => panic!("Attempted to flatten non-object field")
            }
        }};
    }

    let inline = field_attrs.inline.then(|| quote!(def.body.force_inline();));

    let name_str = sanitise_raw_ident(field.ident.as_ref().unwrap());

    let name_str = match (field_attrs.rename.clone(), container_attrs.rename_all) {
        (Some(name), _) => name,
        (None, Some(inflection)) => inflection.apply(&name_str),
        (None, None) => name_str,
    };

    let optional = field_attrs.optional;

    quote! {{
        let mut def = #upsert;

        #inline

        vec![#crate_ref::ObjectField { name: #name_str.to_string(), ty: def, optional: #optional }]
    }}
}

fn parse_tuple_struct_field(
    field: &Field,
    _container_attrs: &ContainerAttr,
    field_attrs: &FieldAttr,
    crate_ref: &TokenStream,
) -> TokenStream {
    let upsert = upsert_def(field, crate_ref);

    // TODO: flatten + optional
    if field_attrs.flatten {
        return quote! {{
            match def.body {
                #crate_ref::DataType::Object(ObjectType { fields, .. }) => fields
                    .into_iter()
                    .map(|of| of.ty)
                    .collect::<Vec<#crate_ref::Typedef>>(),
                _ => panic!("Attempted to flatten non-object field"),
            }
        }};
    }

    let optional = field_attrs.optional.then(|| {
        quote! {
            def.body = #crate_ref::DataType::Nullable(Box::new(def.clone()));
        }
    });

    let inline = field_attrs.inline.then(|| quote!(def.body.force_inline();));

    quote! {{
        let mut def = #upsert;

        #optional

        #inline

        vec![def]
    }}
}

fn parse_enum(
    enum_name_str: &str,
    enum_attrs: &EnumAttr,
    _container_attrs: &ContainerAttr,
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
            let attrs = VariantAttr::from_attrs(&v.attrs).unwrap();

            (v, attrs)
        })
        .filter(|(_, attrs)| !attrs.skip)
        .map(|(variant, attrs)| {
            let variant_name_str = variant.ident.to_string();

            let variant_name_str = match (attrs.rename.clone(), _container_attrs.rename_all) {
                (Some(name), _) => name,
                (None, Some(inflection)) => inflection.apply(&variant_name_str),
                (None, None) => variant_name_str,
            };

            match &variant.fields {
                Fields::Unit => {
                    quote!(#crate_ref::EnumVariant::Unit(#variant_name_str.to_string()))
                }
                Fields::Unnamed(fields) => {
                    let fields = fields
                        .unnamed
                        .iter()
                        .map(|f| upsert_def(f, crate_ref))
                        .collect::<Vec<_>>();

                    quote!(#crate_ref::EnumVariant::Unnamed(#crate_ref::TupleType {
                        name: #variant_name_str.to_string(),
                        fields: vec![#(#fields),*],
                        inline: true,
                    }))
                }
                Fields::Named(fields) => {
                    let fields = fields
                        .named
                        .iter()
                        .map(|f| to_object_field(f, crate_ref))
                        .collect::<Vec<_>>();

                    quote!(#crate_ref::EnumVariant::Named(#crate_ref::ObjectType {
                        name: #variant_name_str.to_string(),
                        fields: vec![#(#fields),*],
                        inline: true,
                        tag: None
                    }))
                }
            }
        })
        .collect::<Vec<_>>();

    let repr = match enum_attrs.tagged().unwrap() {
        Tagged::Externally => quote!(External),
        Tagged::Untagged => quote!(Untagged),
        Tagged::Adjacently { tag, content } => {
            quote!(Adjacent { tag: #tag.to_string(), content: #content.to_string() })
        }
        Tagged::Internally { tag } => {
            quote!(Internal { tag: #tag.to_string() })
        }
    };

    quote!(#crate_ref::DataType::Enum(#crate_ref::EnumType {
        name: #enum_name_str.to_string(),
        inline: false,
        variants: vec![#(#variants),*],
        repr: #crate_ref::EnumRepr::#repr,
    }))
}

fn to_object_field(f: &Field, crate_ref: &TokenStream) -> TokenStream {
    let ident = f.ident.as_ref().unwrap();
    let ty = upsert_def(f, crate_ref);

    let name = sanitise_raw_ident(ident);

    quote! {
        #crate_ref::ObjectField {
            name: #name.into(),
            ty: #ty,
            optional: false
        }
    }
}

fn upsert_def(f: &Field, crate_ref: &TokenStream) -> TokenStream {
    let ty = &f.ty;

    quote! {
        if let Some(def) = defs.get(&std::any::TypeId::of::<#ty>()) {
            def.clone()
        } else {
            let def = <#ty as #crate_ref::Type>::def(defs);
            defs.insert(std::any::TypeId::of::<#ty>(), def.clone());
            def
        }
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
