use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, ConstParam, GenericArgument, GenericParam, Generics, Ident, LifetimeDef,
    PathArguments, Type, TypeArray, TypeParam, TypePtr, TypeReference, TypeSlice, WhereClause,
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
                    &el,
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
                let #var_ident = generics.get(#i).map(Clone::clone).unwrap_or(
                    <#generic_ident as #crate_ref::Type>::#method(
                        #crate_ref::DefOpts {
                            parent_inline: #inline,
                            type_map: opts.type_map
                        },
                        &[#crate_ref::DataType::Generic(
                            stringify!(#type_ident).to_string()
                        )]
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
            .filter_map(|(i, arg)| match arg {
                GenericArgument::Type(ty) => Some((i, ty)),
                _ => todo!("one"),
            })
            .collect(),
        PathArguments::None => vec![],
        _ => panic!("Only angle bracketed generics are supported!"),
    };

    let generic_vars = generic_args.iter().map(|(i, path)| {
        construct_datatype(
            format_ident!("{}_{}", &var_ident, i),
            &path,
            &generic_idents,
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
