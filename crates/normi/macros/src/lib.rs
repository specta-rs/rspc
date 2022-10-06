use attrs::FieldAttrs;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Index};

use crate::attrs::StructAttrs;

#[macro_use]
mod utils;
mod attrs;

#[proc_macro_derive(Object, attributes(normi))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("normi");
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);
    let args = StructAttrs::from_attrs(&attrs).unwrap();
    let type_name = args.rename.unwrap_or(ident.to_string());
    let normalised_ident = format_ident!("Normalised{}", ident);

    match data {
        Data::Struct(data) => {
            let mut fields = data.fields.iter().peekable();
            let is_tuple_struct = fields.peek().map(|v| v.ident.is_some()).expect("normi::Object requires at least one fields");

            let mut id_fields = Vec::new();
            for (i, field) in fields.enumerate() {
                let attrs = FieldAttrs::from_attrs(&field.attrs).unwrap();
                
                attrs.id.then(|| id_fields.push(
                    match &field.ident {
                        Some(ident) => quote!(self.#ident),
                        None => {
                            let i = Index::from(i);
                            quote!(self.#i)
                        }
                    })
                );
            }
            
            let mut id_fields = id_fields.into_iter().peekable();
            let _ = id_fields.peek().ok_or_else(|| panic!("normi::Object must have an id field set. Ensure you add `#[normi(id)]`"));
            let id_impl = id_fields.peek().map(|field| quote! ( #crate_name::internal::to_value(&#field).unwrap() )).unwrap_or(quote! ( #crate_name::internal::to_value(&[#(&#id_fields),*]).unwrap() ));

            let object_impl_decl = {
                let fields = match is_tuple_struct {
                    true => {
                        let fields = data.fields
                        .iter()
                        .map(|f| {
                            let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();
                            let ident = f.ident.clone().unwrap();
                            if attrs.refr {
                                quote!( #ident: self.#ident.normalize()? )
                            } else {
                                quote! ( #ident: self.#ident )
                            }
                        })
                        .collect::<Vec<_>>();
                        
                        quote!( #(#fields),* )
                    },
                    false => {
                        let fields = data.fields
                            .iter()
                            .enumerate()
                            .map(|(i, f)| {
                                let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();
                                if attrs.refr {
                                    quote!( self.#i.normalize()? )
                                } else {
                                    let i = Index::from(i);
                                    quote! ( self.#i )
                                }
                            })
                            .collect::<Vec<_>>();
                        if fields.len() == 1 {
                            panic!("You must have more than one field on a tuple struct!");
                        }
                        
                        quote!( data: (#(#fields),*) )
                    }
                };
                
                quote! {
                    impl #crate_name::Object for #ident {
                        type NormalizedResult = #normalised_ident;

                        fn type_name() -> &'static str {
                            #type_name
                        }

                        fn id(&self) -> Result<#crate_name::internal::Value, #crate_name::internal::Error> {
                            Ok(#id_impl)
                        }

                        fn normalize(self) -> Result<Self::NormalizedResult, #crate_name::internal::Error> {
                            pub use #crate_name::Object;

                            Ok(#normalised_ident {
                                __type: Self::type_name(),
                                __id: self.id()?,
                                #fields
                            })
                        }
                    }
                }
            };

            let normalized_struct_decl = {
                let fields = data.fields.into_iter().map(|f| {
                    let ty = f.ty;
                    let vis = f.vis;
                    let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();

                    match &f.ident {
                        Some(ident) => {
                            if attrs.refr {
                                quote!( #vis #ident: <#ty as #crate_name::Object>::NormalizedResult )
                            } else {
                                quote!( #vis #ident: #ty )
                            }
                        },
                        None => quote!( #ty ),
                    }
                });

                let fields = match is_tuple_struct {
                    true => quote! ( #(#fields),* ),
                    false => quote! ( data: (#(#fields),*) ),
                };
            
                quote! {
                    #[derive(#crate_name::internal::Serialize, #crate_name::internal::Type)]
                    pub struct #normalised_ident {
                        pub __type: &'static str,
                        pub __id: #crate_name::internal::Value,
                        #fields
                    }
                }
            };

            quote! {
                #normalized_struct_decl
                #object_impl_decl
            }
            
        }
        Data::Enum(_) => panic!("TODO: enums not supported"), // TODO
        Data::Union(_) => panic!("normi::Object can't be derived for unions!"),
    }.into()
}
