use syn::{Attribute, Result};

use crate::utils::{filter_attrs, AttributeParser};

#[derive(Default)]
pub struct StructAttr {
    pub transparent: bool,
}

impl_parse! {
    StructAttr(attr, out) {
        "transparent" => out.transparent = true,
    }
}

impl StructAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs(filter_attrs("specta", attrs), &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs(filter_attrs("serde", attrs), &mut result)?;
        Self::try_from_attrs(filter_attrs("repr", attrs), &mut result)?; // To handle `#[repr(transparent)]`
        Ok(result)
    }
}
