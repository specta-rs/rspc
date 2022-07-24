mod attrs;

use attrs::{
    DeriveContainerAttrs, DeriveEnumAttrs, DeriveEnumVariantAttrs, DeriveStructFieldAttrs,
};
use darling::{FromDeriveInput, FromField, FromVariant};
use quote::{__private::TokenStream, format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Ident};

#[proc_macro_derive(Type, attributes(specta))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input);

    let container_attrs = DeriveContainerAttrs::from_derive_input(&derive_input).unwrap();

    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = &derive_input;

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
        Data::Struct(data) => parse_struct(&name_str, &container_attrs, &crate_ref, data),
        Data::Enum(data) => {
            let enum_attrs = DeriveEnumAttrs::from_derive_input(&derive_input).unwrap();

            parse_enum(&name_str, &container_attrs, &enum_attrs, &crate_ref, data)
        }
        Data::Union(_) => panic!("Type 'Union' is not supported by specta!"),
    };

    quote! {
        impl #crate_name::Type for #ident {
            fn def(defs: &mut #crate_name::TypeDefs) -> #crate_name::Typedef {
                #crate_name::Typedef {
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
    container_attrs: &DeriveContainerAttrs,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> TokenStream {
    let inline = container_attrs.inline;

    match &data.fields {
        Fields::Unit => quote!(#crate_ref::BodyDefinition::Tuple {
            name: #struct_name_str.to_string(),
            inline: true,
            fields: vec![],
        }),
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| {
                let attrs = DeriveStructFieldAttrs::from_field(f).unwrap();

                parse_tuple_struct_field(f, &container_attrs, attrs, crate_ref)
            });

            quote!(#crate_ref::BodyDefinition::Tuple {
                name: #struct_name_str.to_string(),
                inline: #inline,
                fields: (vec![#(#fields),*] as Vec<Vec<_>>)
                    .into_iter()
                    .flatten()
                    .collect(),
            })
        }
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let attrs = DeriveStructFieldAttrs::from_field(f).unwrap();

                parse_named_struct_field(f, &container_attrs, attrs, crate_ref)
            });

            quote!(#crate_ref::BodyDefinition::Object {
                name: #struct_name_str.to_string(),
                inline: #inline,
                fields: (vec![#(#fields),*] as Vec<Vec<_>>)
                    .into_iter()
                    .flatten()
                    .collect()
            })
        }
    }
}

fn parse_named_struct_field(
    field: &Field,
    _container_attrs: &DeriveContainerAttrs,
    field_attrs: DeriveStructFieldAttrs,
    crate_ref: &TokenStream,
) -> TokenStream {
    let upsert = upsert_def(field, crate_ref);

    // TODO: flatten + optional
    let flatten = field_attrs.flatten.then(|| {
        quote! {
            match def.body {
                #crate_ref::BodyDefinition::Object { fields, .. } => return fields.into_iter().map(|f| f),
                _ => panic!("Attempted to flatten non-object field {} of struct {}");
            }
        }
    });

    let optional = field_attrs.optional.then(|| {
        quote! {
            def.body = #crate_ref::BodyDefinition::Nullable(Box::new(def.body));
        }
    });

    let inline = field_attrs.inline.then(|| quote!(def.body.force_inline();));

    let name_str = field_attrs
        .rename
        .unwrap_or(field.ident.as_ref().unwrap().to_string());

    quote! {{
        let mut def = #upsert;

        #flatten

        #optional

        #inline

        vec![#crate_ref::ObjectField { name: #name_str.to_string(), ty: def }]
    }}
}

fn parse_tuple_struct_field(
    field: &Field,
    container_attrs: &DeriveContainerAttrs,
    field_attrs: DeriveStructFieldAttrs,
    crate_ref: &TokenStream,
) -> TokenStream {
    let upsert = upsert_def(field, crate_ref);

    // TODO: flatten + optional
    let flatten = field_attrs.flatten.then(|| {
        quote! {
            match def.body {
                #crate_ref::BodyDefinition::Object { fields, .. } => return fields.into_iter().map(|f| f.ty).collect(),
                _ => panic!("Attempted to flatten non-object field {} of struct {}");
            }
        }
    });

    let optional = field_attrs.optional.then(|| {
        quote! {
            def.body = #crate_ref::BodyDefinition::Nullable(Box::new(def.body));
        }
    });

    let inline = field_attrs.inline.then(|| quote!(def.body.force_inline();));

    quote! {{
        let mut def = #upsert;

        #flatten

        #optional

        #inline

        vec![def]
    }}
}

fn parse_enum(
    enum_name_str: &str,
    _container_attrs: &DeriveContainerAttrs,
    enum_attrs: &DeriveEnumAttrs,
    crate_ref: &TokenStream,
    data: &DataEnum,
) -> TokenStream {
    if data.variants.len() == 0 {
        panic!("Enum {} has 0 fields!", enum_name_str);
    }

    let variants = data
        .variants
        .iter()
        .map(|v| {
            let attrs = DeriveEnumVariantAttrs::from_variant(v).unwrap();

            (v, attrs)
        })
        .filter(|(_, attrs)| !attrs.skip)
        .map(|(variant, attrs)| {
            let variant_name_str = attrs.rename.unwrap_or(variant.ident.to_string());

            match &variant.fields {
                Fields::Unit => {
                    quote!(#crate_ref::EnumVariant::Unit(#variant_name_str.to_string()))
                }
                Fields::Unnamed(fields) => {
                    let fields = fields.unnamed.iter().map(|f| upsert_def(f, crate_ref));

                    quote!(#crate_ref::EnumVariant::Unnamed(
                        #variant_name_str.to_string(),
                        #crate_ref::BodyDefinition::Tuple {
                            name: #variant_name_str.to_string(),
                            inline: true,
                            fields: vec![#(#fields),*],
                        }
                    ))
                }
                Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|f| to_object_field(f, crate_ref));

                    quote!(#crate_ref::EnumVariant::Named(
                        #variant_name_str.to_string(),
                        #crate_ref::BodyDefinition::Object {
                            name: #variant_name_str.to_string(),
                            fields: vec![#(#fields),*],
                            inline: true,
                        }
                    ))
                }
            }
        });

    let repr = match (&enum_attrs.tag, &enum_attrs.content, enum_attrs.untagged) {
        (None, None, true) => quote!(Untagged),
        (Some(tag), None, false) => quote!(Internal {
            tag: #tag.to_string()
        }),
        (Some(tag), Some(content), false) => quote!(Adjacent {
            tag: #tag.to_string(),
            content: #content.to_string()
        }),
        (None, None, false) => quote!(External),
        (_, _, _) => panic!("Enum {enum_name_str} has invalid representation"),
    };

    quote!(#crate_ref::BodyDefinition::Enum {
        name: #enum_name_str.to_string(),
        inline: false,
        variants: vec![#(#variants),*],
        repr: #crate_ref::EnumRepr::#repr,
    })
}

fn to_object_field(f: &Field, crate_ref: &TokenStream) -> TokenStream {
    let ident = f.ident.as_ref().unwrap();
    let ty = upsert_def(f, crate_ref);

    quote! {
        #crate_ref::ObjectField {
            name: stringify!(#ident).into(),
            ty: #ty
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
