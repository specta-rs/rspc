use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{__private::TokenStream as TokenStream2, format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Ident};

#[derive(FromDeriveInput)]
#[darling(attributes(specta))]
struct DeriveTypeArgs {
    rename: Option<String>,
}

#[proc_macro_derive(Type, attributes(specta))]
pub fn derive_type(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("specta");
    let derive_input = parse_macro_input!(input);

    let args = DeriveTypeArgs::from_derive_input(&derive_input).unwrap();

    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = &derive_input;

    // TODO: Deal with struct or enum with generics
    // TODO: Struct attributes -> Renaming field, etc + Serde compatibility

    let body = match data {
        Data::Struct(data) => parse_struct(&crate_name, data),
        Data::Enum(data) => parse_enum(&crate_name, &ident, data),
        Data::Union(_) => panic!("Type 'Union' is not supported by specta!"),
    };

    let name_str = args.rename.unwrap_or(ident.to_string());

    quote! {
        impl #crate_name::Type for #ident {
            fn def(defs: &mut #crate_name::TypeDefs) -> #crate_name::Typedef {
                #crate_name::Typedef {
                    name: #name_str.to_string(),
                    primitive: false,
                    type_id: std::any::TypeId::of::<#ident>(),
                    body: #body,
                }
            }
        }
    }
    .into()
}

fn to_specta_field(f: &Field, crate_name: &Ident) -> TokenStream2 {
    let ident = f.ident.as_ref().unwrap();
    let ty = &f.ty;

    quote! {
        #crate_name::Field {
            name: stringify!(#ident).into(),
            ty: if let Some(def) =
               defs.get(&std::any::TypeId::of::<#ty>()) {
                def.clone()
            } else {
                let def = <#ty as #crate_name::Type>::def(defs);
                defs.insert(std::any::TypeId::of::<#ty>(), def.clone());
                def
            },
        }
    }
}

fn get_specta_typedef(f: &Field, crate_name: &Ident) -> TokenStream2 {
    let ty = &f.ty;
    quote! {
        if let Some(def) = defs.get(&std::any::TypeId::of::<#ty>()) {
            def.clone()
        } else {
            let def = <#ty as #crate_name::Type>::def(defs);
            defs.insert(std::any::TypeId::of::<#ty>(), def.clone());
            def
        }
    }
}

fn parse_struct(crate_name: &Ident, data: &DataStruct) -> TokenStream2 {
    match &data.fields {
        Fields::Unit => quote!(#crate_name::BodyDefinition::Tuple(vec![])),
        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .map(|f| get_specta_typedef(f, crate_name));

            quote!(#crate_name::BodyDefinition::Tuple(vec![#(#fields),*]))
        }
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| to_specta_field(f, crate_name));

            quote!(#crate_name::BodyDefinition::Object(vec![#(#fields),*]))
        }
    }
}

fn parse_enum(crate_name: &Ident, enum_name: &Ident, data: &DataEnum) -> TokenStream2 {
    if data.variants.len() == 0 {
        panic!("Enum {} has 0 fields!", enum_name.to_string());
    }

    let variants = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = quote!(stringify!(#variant_name).to_string());

        match &variant.fields {
            Fields::Unit => {
                quote!(#crate_name::EnumVariant::Unit(#variant_name_str))
            }
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .map(|f| get_specta_typedef(f, crate_name));

                quote!(::#crate_name::EnumVariant::Unnamed(
                    #variant_name_str,
                    ::#crate_name::BodyDefinition::Tuple(vec![#(#fields),*])
                ))
            }
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(|f| to_specta_field(f, crate_name));

                quote!(::#crate_name::EnumVariant::Named(
                    #variant_name_str,
                    ::#crate_name::BodyDefinition::Object(vec![#(#fields),*])
                ))
            }
        }
    });

    quote!(#crate_name::BodyDefinition::Enum(vec![#(#variants),*]))
}
