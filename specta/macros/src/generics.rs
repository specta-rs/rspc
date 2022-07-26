use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, ConstParam, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, GenericParam,
    Generics, Ident, LifetimeDef, TypeParam, WhereClause,
};

// Code copied from ts-rs. Thanks to it's original author!
// generate start of the `impl TS for #ty` block, up to (excluding) the open brace
pub fn generate_impl(crate_name: &TokenStream, ty: &Ident, generics: &Generics) -> TokenStream {
    use GenericParam::*;

    let bounds = generics.params.iter().map(|param| match param {
        Type(TypeParam {
            ident,
            colon_token,
            bounds,
            ..
        }) => quote!(#ident #colon_token #bounds),
        Lifetime(LifetimeDef {
            lifetime,
            colon_token,
            bounds,
            ..
        }) => quote!(#lifetime #colon_token #bounds),
        Const(ConstParam {
            const_token,
            ident,
            colon_token,
            ty,
            ..
        }) => quote!(#const_token #ident #colon_token #ty),
    });
    let type_args = generics.params.iter().map(|param| match param {
        Type(TypeParam { ident, .. }) | Const(ConstParam { ident, .. }) => quote!(#ident),
        Lifetime(LifetimeDef { lifetime, .. }) => quote!(#lifetime),
    });

    let where_bound = add_type_to_where_clause(crate_name, generics);
    quote!(impl <#(#bounds),*> #crate_name::Type for #ty <#(#type_args),*> #where_bound)
}

pub fn generic_refs(generics: &Generics, crate_ref: &TokenStream) -> TokenStream {
    let generics = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => {
                let ident = &ty.ident;
                Some(quote!(<#ident as #crate_ref::Type>::def(defs)))
            }
            GenericParam::Lifetime(_) => None,
            GenericParam::Const(_) => None, // TODO: Support const generics
        })
        .collect::<Vec<_>>();

    quote! { vec![#(#generics),*] }
}

pub fn type_ident_with_generics(ident: &Ident, generics: &Generics) -> TokenStream {
    let generics = generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                quote! { #ident }
            }
            GenericParam::Lifetime(param) => {
                let ident = &param.lifetime;
                quote! { #ident }
            }
            GenericParam::Const(_) => panic!("const generics are not supported by specta!"), // TODO: Support const generics
        })
        .collect::<Vec<_>>();

    quote! { #ident<#(#generics),*> }
}

// Code copied from ts-rs. Thanks to it's original author!
fn add_type_to_where_clause(crate_name: &TokenStream, generics: &Generics) -> Option<WhereClause> {
    let generic_types = generics
        .params
        .iter()
        .filter_map(|gp| match gp {
            GenericParam::Type(ty) => Some(ty.ident.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if generic_types.is_empty() {
        return generics.where_clause.clone();
    }
    match generics.where_clause {
        None => Some(parse_quote! { where #( #generic_types : #crate_name::Type + 'static ),* }),
        Some(ref w) => {
            let bounds = w.predicates.iter();
            Some(
                parse_quote! { where #(#bounds,)* #( #generic_types : #crate_name::Type + 'static ),* },
            )
        }
    }
}
