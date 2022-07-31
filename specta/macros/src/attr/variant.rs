use syn::{Attribute, Result};

use super::parse_assign_str;
use crate::utils::parse_attrs;

#[derive(Default)]
pub struct VariantAttr {
    pub rename: Option<String>,
    pub skip: bool,
}

#[cfg(feature = "serde")]
#[derive(Default)]
pub struct SerdeVariantAttr(VariantAttr);

impl VariantAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a| result.merge(a));
        #[cfg(feature = "serde")]
        crate::utils::parse_serde_attrs::<SerdeVariantAttr>(attrs).for_each(|a| result.merge(a.0));
        Ok(result)
    }

    fn merge(&mut self, VariantAttr { rename, skip }: VariantAttr) {
        self.rename = self.rename.take().or(rename);
        self.skip = self.skip || skip;
    }
}

impl_parse! {
    VariantAttr(input, out) {
        "rename" => out.rename = Some(parse_assign_str(input)?),
        "skip" => out.skip = true,
    }
}

#[cfg(feature = "serde")]
impl_parse! {
    SerdeVariantAttr(input, out) {
        "rename" => out.0.rename = Some(parse_assign_str(input)?),
        "skip" => out.0.skip = true,
        "skip_serializing" => out.0.skip = true,
        "skip_deserializing" => out.0.skip = true,
        // parse #[serde(default)] to not emit a warning
        "default" => {
            use syn::Token;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                parse_assign_str(input)?;
            }
        },
    }
}
