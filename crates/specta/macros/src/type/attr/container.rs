use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Result;

use crate::utils::{Inflection, MetaAttr};

#[derive(Default, Clone)]
pub struct ContainerAttr {
    pub rename_all: Option<Inflection>,
    pub rename: Option<TokenStream>,
    pub tag: Option<String>,
    pub crate_name: Option<String>,
    pub inline: bool,
    pub remote: Option<String>,
    pub doc: Vec<String>,
}

impl_parse! {
    ContainerAttr(attr, out) {
        "rename_all" => out.rename_all = out.rename_all.take().or(Some(attr.pass_inflection()?)),
        "rename" => {
            let attr = attr.pass_string()?;
            out.rename = out.rename.take().or_else(|| Some({
                let name = crate::r#type::unraw_raw_ident(&quote::format_ident!("{}", attr));
                quote::quote!( #name )
            }))
        },
        "rename_to_value" => {
            let attr = attr.pass_path()?;
            out.rename = out.rename.take().or_else(|| Some({
                let expr = attr.to_token_stream();
                quote::quote!( #expr )
            }))
        },
        "tag" => out.tag = out.tag.take().or(Some(attr.pass_string()?)),
        "crate" => {
            if attr.root_ident() == "specta" {
                out.crate_name = out.crate_name.take().or(Some(attr.pass_string()?));
            }
        },
        "inline" => out.inline = true,
        "remote" => out.remote = out.remote.take().or(Some(attr.pass_string()?)),
        "doc" => {
            if attr.tag().as_str() == "doc" {
                out.doc.push(attr.pass_string()?);
            }
        }
    }
}

impl ContainerAttr {
    pub fn from_attrs(attrs: &mut Vec<MetaAttr>) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs("specta", attrs, &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs("serde", attrs, &mut result)?;
        Self::try_from_attrs("doc", attrs, &mut result)?;
        Ok(result)
    }
}
