use quote::format_ident;
use syn::{Attribute, Ident, Lit, MetaNameValue, Result, Type};

pub fn filter_attrs<'a>(
    path: &'static str,
    attrs: &'a [Attribute],
) -> impl Iterator<Item = &'a Attribute> + 'a {
    attrs.iter().filter(|a| a.path.is_ident(path))
}

pub struct AttributeParser(pub MetaNameValue);

impl AttributeParser {
    pub fn tag(&self) -> String {
        self.0.path.get_ident().unwrap().to_string()
    }

    pub fn pass_string(&self) -> Result<String> {
        match &self.0.lit {
            Lit::Str(string) => Ok(string.value()),
            lit => Err(syn::Error::new_spanned(
                lit,
                "specta: expected string literal",
            )),
        }
    }

    pub fn pass_type(&self) -> Result<Type> {
        todo!();
        // match &self.0.lit {
        //     Lit::Str(string) => Ok(string.value()),

        //     Lit::
        //     lit => Err(syn::Error::new_spanned(
        //         lit,
        //         "specta: expected string literal",
        //     )),
        // }

        // println!("{:?}", self.0.lit);
        // Ok(Type::Verbatim(quote::quote!(TODO)))
    }

    pub fn pass_inflection(&self) -> Result<Inflection> {
        match &self.0.lit {
            Lit::Str(lit) => Ok(match lit.value().to_lowercase().replace('_', "").as_str() {
                "lowercase" => Inflection::Lower,
                "uppercase" => Inflection::Upper,
                "camelcase" => Inflection::Camel,
                "snakecase" => Inflection::Snake,
                "pascalcase" => Inflection::Pascal,
                "screamingsnakecase" => Inflection::ScreamingSnake,
                _ => {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "specta: string literal contains un unsupported inflection",
                    ))
                }
            }),
            lit => Err(syn::Error::new_spanned(
                lit,
                "specta: expected string literal containing an inflection",
            )),
        }
    }
}

macro_rules! impl_parse {
    ($i:ident ($attr_paser:ident, $out:ident) { $($k:pat => $e:expr),* $(,)? }) => {
        impl $i {
            fn try_from_attrs<'a>(
                attrs: impl Iterator<Item = &'a Attribute>,
                $out: &mut Self,
            ) -> syn::Result<()> {
                for attr in attrs {
                    let meta = attr.parse_meta()?;

                    let mut handle = |$attr_paser: AttributeParser| {
                        let tag = $attr_paser.tag();
                        match tag.as_str() {
                            $($k => $e,)*
                            #[allow(unreachable_patterns)]
                            _ => {
                                // Throw error if the attribute is not matched by Specta
                                // We ignore errors in any other attributes because their author could change them at any time.
                                if attr.path.is_ident("specta") {
                                    return Err(syn::Error::new_spanned(
                                        attr,
                                        format!("specta: found unknown Specta attribute '{}'", tag),
                                    ));
                                }
                            }
                        }
                        Ok(())
                    };

                    match meta {
                        // Specta or serde attributes
                        syn::Meta::List(list) => {
                            for nested in list.nested {
                                match nested {
                                    syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => handle(AttributeParser(nv))?,
                                    nested => {
                                        use quote::ToTokens;
                                        return Err(syn::Error::new_spanned(
                                            nested.clone(),
                                            format!(
                                                "specta: expected `NestedMeta::Meta`. Found '{}'.",
                                                nested.to_token_stream().to_string()
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                        // Doc comments
                        syn::Meta::NameValue(nv) => handle(AttributeParser(nv))?,
                        _ => return Err(syn::Error::new_spanned(
                            attr,
                            "specta: unexpected found `Meta::Path`.",
                        )),
                    }
                }

                Ok(())
            }
        }
    };
}

pub fn unraw_raw_ident(ident: &Ident) -> String {
    let ident = ident.to_string();
    if ident.starts_with("r#") {
        ident.trim_start_matches("r#").to_owned()
    } else {
        ident
    }
}

#[derive(Copy, Clone)]
pub enum Inflection {
    Lower,
    Upper,
    Camel,
    Snake,
    Pascal,
    ScreamingSnake,
}

impl Inflection {
    pub fn apply(self, string: &str) -> String {
        use inflector::Inflector;

        match self {
            Inflection::Lower => string.to_lowercase(),
            Inflection::Upper => string.to_uppercase(),
            Inflection::Camel => string.to_camel_case(),
            Inflection::Snake => string.to_snake_case(),
            Inflection::Pascal => string.to_pascal_case(),
            Inflection::ScreamingSnake => string.to_screaming_snake_case(),
        }
    }
}

pub fn format_fn_wrapper(function: &Ident) -> Ident {
    format_ident!("__specta__fn__{}", function)
}
