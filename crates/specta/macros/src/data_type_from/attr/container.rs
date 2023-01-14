use syn::{Attribute, Result};

use crate::utils::{filter_attrs, AttributeParser};

#[derive(Default)]
pub struct ContainerAttr {
    pub crate_name: Option<String>,
}

impl_parse! {
    ContainerAttr(attr, out) {
        "crate" => out.crate_name = out.crate_name.take().or(Some(attr.pass_string()?)),
    }
}

impl ContainerAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs(filter_attrs("specta", attrs), &mut result)?;
        Ok(result)
    }
}
