use darling::FromDeriveInput;
use quote::{__private::TokenStream, format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Ident};

#[derive(FromDeriveInput)]
#[darling(attributes(specta))]
struct DeriveTypeArgs {
    rename: Option<String>,
    #[darling(rename = "crate")]
    crate_name: Option<String>,
    #[darling(default)]
    inline: bool
}

#[proc_macro_derive(Type, attributes(specta))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input);

    let args = DeriveTypeArgs::from_derive_input(&derive_input).unwrap();

    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = &derive_input;

    let crate_name: TokenStream = args
        .crate_name
        .unwrap_or_else(|| "specta".to_string())
        .parse()
        .unwrap();

    let crate_ref = quote!(::#crate_name);
    // TODO: Deal with struct or enum with generics
    // TODO: Struct attributes -> Renaming field, etc + Serde compatibility

    let name_str = args.rename.unwrap_or(ident.to_string());

    let body = match data {
        Data::Struct(data) => parse_struct(&name_str, args.inline, &crate_ref, data),
        Data::Enum(data) => parse_enum(&name_str, args.inline, &crate_ref, data),
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
fn parse_struct(struct_name: &str, inline: bool, crate_ref: &TokenStream, data: &DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Unit => quote!(#crate_ref::BodyDefinition::Tuple {
            name: None,
            fields: vec![],
        }),
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| upsert_def(f, crate_ref));

            quote!(#crate_ref::BodyDefinition::Tuple {
                name: Some(#struct_name.to_string()),
                fields: vec![#(#fields),*],
            })
        }
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| to_object_field(f, crate_ref));

            quote!(#crate_ref::BodyDefinition::Object {
                name: #struct_name.to_string(),
                inline: #inline,
                fields: vec![#(#fields),*]
            })
        }
    }
}

fn parse_enum(enum_name: &str, inline: bool, crate_ref: &TokenStream, data: &DataEnum) -> TokenStream {
    if data.variants.len() == 0 {
        panic!("Enum {} has 0 fields!", enum_name);
    }

    let variants = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = quote!(stringify!(#variant_name).to_string());

        match &variant.fields {
            Fields::Unit => {
                quote!(#crate_ref::EnumVariant::Unit(#variant_name_str))
            }
            Fields::Unnamed(fields) => {
                let fields = fields.unnamed.iter().map(|f| upsert_def(f, crate_ref));

                quote!(#crate_ref::EnumVariant::Unnamed(
                    #variant_name_str,
                    #crate_ref::BodyDefinition::Tuple {
                        name: None,
                        fields: vec![#(#fields),*],
                    }
                ))
            }
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(|f| to_object_field(f, crate_ref));

                quote!(#crate_ref::EnumVariant::Named(
                    #variant_name_str,
                    #crate_ref::BodyDefinition::Object {
                        name: #variant_name_str,
                        fields: vec![#(#fields),*],
                        inline: true,
                    }
                ))
            }
        }
    });

    quote!(#crate_ref::BodyDefinition::Enum {
        name: #enum_name.to_string(),
        inline: #inline,
        variants: vec![#(#variants),*]
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
