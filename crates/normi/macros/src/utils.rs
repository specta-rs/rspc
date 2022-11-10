use syn::{
    parse::{Parse, ParseStream},
    Attribute, Error, Lit, Result, Token,
};

macro_rules! syn_err {
    ($l:literal $(, $a:expr)*) => {
        syn_err!(::proc_macro2::Span::call_site(); $l $(, $a)*)
    };
    ($s:expr; $l:literal $(, $a:expr)*) => {
        return Err(::syn::Error::new($s, format!($l $(, $a)*)))
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

pub fn parse_attrs<'a, A>(attrs: &'a [Attribute]) -> syn::Result<impl Iterator<Item = A>>
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

pub fn parse_assign_str(input: ParseStream) -> Result<String> {
    input.parse::<Token![=]>()?;
    match Lit::parse(input)? {
        Lit::Str(string) => Ok(string.value()),
        other => Err(Error::new(other.span(), "expected string")),
    }
}
