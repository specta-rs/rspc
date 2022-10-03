use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field};

macro_rules! syn_err {
    ($l:literal $(, $a:expr)*) => {
        syn_err!(proc_macro2::Span::call_site(); $l $(, $a)*)
    };
    ($s:expr; $l:literal $(, $a:expr)*) => {
        return Err(syn::Error::new($s, format!($l $(, $a)*)))
    };
}

macro_rules! impl_parse {
    ($i:ident ($input:ident, $out:ident) { $($k:pat => $e:expr),* $(,)? }) => {
        impl std::convert::TryFrom<&syn::Attribute> for $i {
            type Error = syn::Error;

            fn try_from(attr: &syn::Attribute) -> syn::Result<Self> { attr.parse_args() }
        }

        impl syn::parse::Parse for $i {
            fn parse($input: syn::parse::ParseStream) -> syn::Result<Self> {
                #[allow(warnings)]
                let mut $out = $i::default();
                loop {
                    let key: syn::Ident = $input.call(syn::ext::IdentExt::parse_any)?;
                    match &*key.to_string() {
                        $($k => $e,)*
                        #[allow(unreachable_patterns)]
                        _ => syn_err!($input.span(); "unexpected attribute")
                    }

                    match $input.is_empty() {
                        true => break,
                        false => {
                            $input.parse::<syn::Token![,]>()?;
                        }
                    }
                }

                Ok($out)
            }
        }
    };
}

fn parse_attrs<'a, A>(attrs: &'a [Attribute]) -> syn::Result<impl Iterator<Item = A>>
where
    A: TryFrom<&'a Attribute, Error = syn::Error>,
{
    Ok(attrs
        .iter()
        .filter(|a| a.path.is_ident("normi"))
        .map(A::try_from)
        .collect::<syn::Result<Vec<A>>>()?
        .into_iter())
}

#[derive(Default, Clone, Debug)]
struct FieldAttrs {
    pub id: bool,
    pub refr: bool,
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a: FieldAttrs| {
            result.id = a.id || result.id;
            result.refr = a.refr || result.refr;
        });
        Ok(result)
    }
}

impl_parse! {
    FieldAttrs(input, out) {
        "id" => out.id = true,
        "refr" => out.refr = true,
    }
}

#[proc_macro_derive(Object, attributes(normi))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let crate_name = format_ident!("normi");
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);
    let normalised_ident = format_ident!("Normalised{}", ident);
    let type_name = ident.to_string(); // TODO: Allow user to override using macro attribute

    // TODO: Build test suite for what I want out of this macro

    // for v in attrs.iter().filter(|a| a.path.is_ident("specta")) {
    //     println!("{:?}", v.to_token_stream().to_string());
    // }

    let mut id_fields = Vec::new();
    let fields = match data {
        Data::Struct(data) => {
            for field in data.fields.iter() {
                match field {
                    Field {
                        ident: ident,
                        attrs,
                        ..
                    } => {
                        let ident = ident.clone().unwrap();
                        let attrs = FieldAttrs::from_attrs(&attrs).unwrap();
                        attrs.id.then(|| id_fields.push(ident));
                    }
                    _ => todo!(),
                }
            }

            data.fields
        }
        _ => todo!(),
    };

    let mut id_fields = id_fields.into_iter().peekable();
    let id_impl = match (id_fields.next(), id_fields.peek().is_some()) {
        (None, false) => panic!("TODO"),
        (None, true) => unreachable!(),
        (Some(field_ident), false) => {
            quote! ( #crate_name::internal::normi_to_json_value(&self.#field_ident).unwrap() )
        }
        (Some(field_ident), true) => {
            quote! ( #crate_name::internal::normi_to_json_value(&[&self.#field_ident, #(&self.#id_fields),*]).unwrap() )
        }
    };

    let field_map = fields
        .iter()
        .map(|f| {
            let ident = f.ident.clone().unwrap();
            let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();

            if attrs.refr {
                quote!( #ident: self.#ident.normalize()? )
            } else {
                quote! ( #ident: self.#ident )
            }
        })
        .collect::<Vec<_>>();

    let field_decls = fields.into_iter().map(|f| {
        let ident = f.ident.unwrap();
        let ty = f.ty;
        let vis = f.vis;

        let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap();

        if attrs.refr {
            quote!( #vis #ident: <#ty as #crate_name::Object>::NormalizedResult )
        } else {
            quote!( #vis #ident: #ty )
        }
    });

    quote! {
        #[derive(#crate_name::internal::NormiSerialize, #crate_name::internal::NormiSpectaType)]
        pub struct #normalised_ident {
            pub __type: &'static str,
            pub __id: #crate_name::internal::NormiSerdeValue,
            #(#field_decls,)*
        }

        impl #crate_name::Object for #ident {
            type NormalizedResult = #normalised_ident;

            fn type_name() -> &'static str {
                #type_name
            }

            fn id(&self) -> #crate_name::internal::NormiResult<#crate_name::internal::NormiSerdeValue> {
                Ok(#id_impl)
            }

            fn normalize(self) -> #crate_name::internal::NormiResult<Self::NormalizedResult> {
                pub use #crate_name::Object;

                Ok(#normalised_ident {
                    __type: Self::type_name(),
                    __id: self.id()?,
                    #(#field_map),*
                })
            }
        }
    }
    .into()
}
