use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, ConstParam, GenericArgument, GenericParam, Generics, Ident, LifetimeDef,
    PathArguments, Type, TypeArray, TypeParam, TypePtr, TypeReference, TypeSlice, WhereClause,
};

pub fn generics_with_ident_and_bounds_only(generics: &Generics) -> Option<TokenStream> {
    (!generics.params.is_empty())
        .then(|| {
            use GenericParam::*;
            generics.params.iter().map(|param| match param {
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
            })
        })
        .map(|gs| quote!(<#(#gs),*>))
}

pub fn generics_with_ident_only(generics: &Generics) -> Option<TokenStream> {
    (!generics.params.is_empty())
        .then(|| {
            use GenericParam::*;

            generics.params.iter().map(|param| match param {
                Type(TypeParam { ident, .. }) | Const(ConstParam { ident, .. }) => quote!(#ident),
                Lifetime(LifetimeDef { lifetime, .. }) => quote!(#lifetime),
            })
        })
        .map(|gs| quote!(<#(#gs),*>))
}

// Code copied from ts-rs. Thanks to it's original author!
// generate start of the `impl #r#trait for #ty` block, up to (excluding) the open brace
pub fn impl_heading(r#trait: TokenStream, ty: &Ident, generics: &Generics) -> TokenStream {
    let bounds = generics_with_ident_and_bounds_only(generics);
    let type_args = generics_with_ident_only(generics);

    let where_bound = add_type_to_where_clause(&r#trait, generics);
    quote!(impl #bounds #r#trait for #ty #type_args #where_bound)
}

// Code copied from ts-rs. Thanks to it's original author!
pub fn add_type_to_where_clause(ty: &TokenStream, generics: &Generics) -> Option<WhereClause> {
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
        None => Some(parse_quote! { where #( #generic_types : #ty + 'static ),* }),
        Some(ref w) => {
            let bounds = w.predicates.iter();
            Some(parse_quote! { where #(#bounds,)* #( #generic_types : #ty + 'static ),* })
        }
    }
}

pub fn construct_datatype(
    var_ident: Ident,
    ty: &Type,
    generic_idents: &[(usize, &Ident)],
    crate_ref: &TokenStream,
    inline: bool,
) -> TokenStream {
    let method = match inline {
        true => quote!(inline),
        false => quote!(reference),
    };

    let path = match ty {
        Type::Tuple(t) => {
            let elems = t.elems.iter().enumerate().map(|(i, el)| {
                construct_datatype(
                    format_ident!("{}_{}", var_ident, i),
                    el,
                    generic_idents,
                    crate_ref,
                    inline,
                )
            });

            let generic_var_idents = t
                .elems
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("{}_{}", &var_ident, i));

            return quote! {
                #(#elems)*

                let #var_ident = <#ty as #crate_ref::Type>::#method(#crate_ref::DefOpts {
                    parent_inline: false,
                    type_map: opts.type_map
                }, &[#(#generic_var_idents),*]);
            };
        }
        Type::Array(TypeArray { elem, .. }) | Type::Slice(TypeSlice { elem, .. }) => {
            let elem_var_ident = format_ident!("{}_el", &var_ident);
            let elem = construct_datatype(
                elem_var_ident.clone(),
                elem,
                generic_idents,
                crate_ref,
                inline,
            );

            return quote! {
                #elem

                let #var_ident = <#ty as #crate_ref::Type>::#method(#crate_ref::DefOpts {
                    parent_inline: false,
                    type_map: opts.type_map
                }, &[#elem_var_ident]);
            };
        }
        Type::Ptr(TypePtr { elem, .. }) | Type::Reference(TypeReference { elem, .. }) => {
            return construct_datatype(var_ident, elem, generic_idents, crate_ref, inline)
        }
        Type::Path(p) => &p.path,
        _ => panic!("Cannot get path from type {}", quote!(#ty)),
    };

    if let Some(type_ident) = path.get_ident() {
        if let Some((i, generic_ident)) = generic_idents
            .iter()
            .find(|(_, ident)| ident == &type_ident)
        {
            return quote! {
                let #var_ident = generics.get(#i).map(Clone::clone).unwrap_or_else(||
                    <#generic_ident as #crate_ref::Type>::#method(
                        #crate_ref::DefOpts {
                            parent_inline: #inline,
                            type_map: opts.type_map
                        },
                        &[#crate_ref::GenericType(
                            stringify!(#type_ident).to_string()
                        ).into()]
                    )
                );
            };
        }
    }

    let generic_args = match &path.segments.last().unwrap().arguments {
        PathArguments::AngleBracketed(args) => args
            .args
            .iter()
            .enumerate()
            .filter_map(|(i, input)| match input {
                GenericArgument::Type(ty) => Some((i, ty)),
                _ => None,
            })
            .collect(),
        PathArguments::None => vec![],
        _ => panic!("Only angle bracketed generics are supported!"),
    };

    let generic_vars = generic_args.iter().map(|(i, path)| {
        construct_datatype(
            format_ident!("{}_{}", &var_ident, i),
            path,
            generic_idents,
            crate_ref,
            false,
        )
    });

    let generic_var_idents = generic_args
        .iter()
        .map(|(i, _)| format_ident!("{}_{}", &var_ident, i));

    quote! {
        #(#generic_vars)*

        let #var_ident = <#ty as #crate_ref::Type>::#method(
            #crate_ref::DefOpts {
                parent_inline: #inline,
                type_map: opts.type_map
            },
            &[#(#generic_var_idents),*]
        );
    }
}
