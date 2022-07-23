use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{__private::TokenStream as TokenStream2, format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Ident};

#[derive(FromDeriveInput)]
#[darling(attributes(specta))]
struct DeriveTypeArgs {
    rename: Option<String>
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
        Data::Enum(data) => unimplemented!(),
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

fn parse_struct(crate_name: &Ident, data: &DataStruct) -> TokenStream2 {
    if data.fields.len() == 0 {
        return quote! { #crate_name::BodyDefinition::UnitTuple };
    }

    let mut fields = Vec::new();
    if data.fields.iter().next().unwrap().ident.is_some() {
        for field in &data.fields {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            fields.push(quote! {
                   #crate_name::Field {
                   name: stringify!(#ident).into(),
                   ty: if let Some(def) = defs.get(&std::any::TypeId::of::<#ty>()) {
                    def.clone()
                } else {
                    let def = <#ty as #crate_name::Type>::def(defs);
                    defs.insert(std::any::TypeId::of::<#ty>(), def.clone());
                    def
                },
               }
            });
        }

        quote! { #crate_name::BodyDefinition::Object(vec![#(#fields),*]) }
    } else {
        for field in &data.fields {
            let ty = &field.ty;
            fields.push(
                quote! { if let Some(def) = defs.get(&std::any::TypeId::of::<#ty>()) {
                    def.clone()
                } else {
                    let def = <#ty as #crate_name::Type>::def(defs);
                    defs.insert(std::any::TypeId::of::<#ty>(), def.clone());
                    def
                } },
            );
        }

        quote! { #crate_name::BodyDefinition::Enum(vec![#(#fields),*]) }
    }
}
