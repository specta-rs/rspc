use std::convert::TryFrom;

use syn::{Attribute, Error, Ident, Result};

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

pub fn parse_attrs<'a, A>(attrs: &'a [Attribute]) -> Result<impl Iterator<Item = A>>
where
    A: TryFrom<&'a Attribute, Error = Error>,
{
    Ok(attrs
        .iter()
        .filter(|a| a.path.is_ident("specta"))
        .map(A::try_from)
        .collect::<Result<Vec<A>>>()?
        .into_iter())
}

#[cfg(feature = "serde")]
#[allow(unused)]
pub fn parse_serde_attrs<'a, A: TryFrom<&'a Attribute, Error = Error> + 'a>(
    attrs: &'a [Attribute],
) -> impl Iterator<Item = A> + 'a {
    attrs
        .iter()
        .filter(|a| a.path.is_ident("serde"))
        .flat_map(|attr| A::try_from(attr).ok())
}

pub fn unraw_raw_ident(ident: &Ident) -> String {
    let ident = ident.to_string();
    if ident.starts_with("r#") {
        ident.trim_start_matches("r#").to_owned()
    } else {
        ident
    }
}
