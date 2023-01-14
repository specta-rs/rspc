use syn::{Attribute, Result};

use crate::utils::{filter_attrs, AttributeParser, Inflection};

#[derive(Default)]
pub struct VariantAttr {
    pub rename_all: Option<Inflection>,
    pub rename: Option<String>,
    pub skip: bool,
}

impl_parse! {
    VariantAttr(attr, out) {
        "rename_all" => out.rename_all = out.rename_all.take().or(Some(attr.pass_inflection()?)),
        "rename" => out.rename = out.rename.take().or(Some(attr.pass_string()?)),
        "skip" => out.skip = true,
        "skip_serializing" => out.skip = true,
        "skip_deserializing" => out.skip = true,
    }
}

impl VariantAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs(filter_attrs("specta", attrs), &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs(filter_attrs("serde", attrs), &mut result)?;
        Ok(result)
    }
}
