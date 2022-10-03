use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Object, attributes(normi))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("normi");
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let normalised_ident = format_ident!("Normalised{}", ident);
    let type_name = ident.to_string(); // TODO: Allow user to override using macro attribute

    // TODO: Build test suite for what I want out of this macro

    quote! {
        #[derive(#crate_name::internal::NormiSerialize, #crate_name::internal::NormiSpectaType)]
        pub struct #normalised_ident {
            pub __type: &'static str,
            pub __id: #crate_name::internal::NormiSerdeValue,
            #[serde(flatten)]
            pub data: #ident,
        }

        impl #crate_name::Object for #ident {
            type NormalizedResult = #normalised_ident;

            fn type_name() -> &'static str {
                #type_name
            }

            fn id(&self) -> #crate_name::internal::NormiSerdeValue {
                // TODO

                #crate_name::internal::NormiSerdeValue::Null
            }

            fn normalize(self) -> Self::NormalizedResult {
                pub use #crate_name::Object;

                #normalised_ident {
                    __type: Self::type_name(),
                    __id: self.id(),
                    data: self,
                }
            }
        }
    }
    .into()
}
