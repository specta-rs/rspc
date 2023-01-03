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
    let type_name = args.rename.unwrap_or_else(|| ident.to_string());

    match data {
        Data::Struct(data) => {
            let mut fields = data.fields.iter().peekable();
            let is_not_tuple_struct = fields.peek().map(|v| v.ident.is_some()).expect("normi::Object requires at least one fields");

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

            let fields = match is_not_tuple_struct {
                true => {
                    let fields = data.fields
                    .iter()
                    .filter_map(|f| {
                        let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();
                        let ident = f.ident.clone().unwrap();
                        let ident_str = f.ident.clone().unwrap().to_string();
                        if attrs.refr {
                            Some(quote!( #ident_str: self.#ident.normalize(refs)? ))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                    quote!( #(#fields),* )
                },
                false => {
                    // TODO: Support this
                    // let fields = data.fields
                    //     .iter()
                    //     .enumerate()
                    //     .map(|(i, f)| {
                    //         let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();
                    //         if attrs.refr {
                    //             quote!( self.#i.normalize()? )
                    //         } else {
                    //             // let i = Index::from(i);
                    //             quote! ()
                    //         }
                    //     })
                    //     .collect::<Vec<_>>();
                    // if fields.len() == 1 {
                    //     panic!("You must have more than one field on a tuple struct!");
                    // }

                    // quote!( #(#fields),* )
                    panic!("Normi does not currently support normalising tuple structs.");
                }
            };

            quote! {
                impl #crate_name::Object for #ident {
                    fn type_name() -> &'static str {
                        #type_name
                    }

                    fn id(&self) -> Result<#crate_name::internal::Value, #crate_name::internal::Error> {
                        Ok(#id_impl)
                    }

                    fn normalize(self, refs: &mut #crate_name::RefMap) -> Result<#crate_name::internal::Value, #crate_name::internal::Error> {
                        pub use #crate_name::Object;
                        let ty = Self::type_name();
                        let id = self.id()?;

                        refs.insert(
                            #crate_name::ObjectRef {
                                ty,
                                id: id.clone().into(),
                            },
                            #crate_name::internal::to_value(&self)?,
                        );

                        Ok(#crate_name::internal::json!({
                            "$ty": ty,
                            "$id": id,
                            #fields
                        }))
                    }
                }
            }
        }
        Data::Enum(_) => panic!("TODO: enums not supported"), // TODO
        Data::Union(_) => panic!("normi::Object can't be derived for unions!"),
    }.into()
}
