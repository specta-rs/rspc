use syn::{Attribute, Result};

use crate::utils::*;

#[derive(Default, Clone)]
pub struct StructAttr {
    pub transparent: bool,
}

#[cfg(feature = "serde")]
#[derive(Default)]
pub struct SerdeStructAttr(StructAttr);

#[derive(Default)]
pub struct ReprStructAttr(StructAttr);

impl StructAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a| result.merge(a));

        #[cfg(feature = "serde")]
        crate::utils::parse_serde_attrs::<SerdeStructAttr>(attrs).for_each(|a| result.merge(a.0));

        crate::utils::parse_repr_attrs::<ReprStructAttr>(attrs).for_each(|a| result.merge(a.0));

        Ok(result)
    }

    fn merge(&mut self, StructAttr { transparent }: StructAttr) {
        self.transparent |= transparent
    }
}

impl_parse! {
    StructAttr(input, out) {
        "transparent" => out.transparent = true,
    }
}

#[cfg(feature = "serde")]
impl_parse! {
    SerdeStructAttr(input, out) {}
}

impl_parse! {
    ReprStructAttr(input, out) {
        // parse #[serde(default)] to not emit a warning
        "transparent" => {
            out.0.transparent = true
        },
    }
}
