use syn::{Attribute, Result};

use crate::utils::{filter_attrs, AttributeParser, Inflection};

#[derive(Default, Clone)]
pub struct ContainerAttr {
    pub rename_all: Option<Inflection>,
    pub rename: Option<String>,
    pub tag: Option<String>,
    pub crate_name: Option<String>,
    pub inline: bool,
    pub remote: Option<String>,
    pub doc: Vec<String>,
}

impl_parse! {
    ContainerAttr(attr, out) {
        "rename_all" => out.rename_all = out.rename_all.take().or(Some(attr.pass_inflection()?)),
        "rename" => out.rename = out.rename.take().or(Some(attr.pass_string()?)),
        "tag" => out.tag = out.tag.take().or(Some(attr.pass_string()?)),
        "crate" => {
            if attr.tag().as_str() == "specta" {
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
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs(filter_attrs("specta", attrs), &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs(filter_attrs("serde", attrs), &mut result)?;
        Self::try_from_attrs(filter_attrs("doc", attrs), &mut result)?;
        Ok(result)
    }
}
