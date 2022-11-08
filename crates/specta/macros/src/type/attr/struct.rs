use syn::{Attribute, Ident, Result};

use crate::attr::parse_assign_str;

#[derive(Default, Clone)]
pub struct StructAttr {}

#[cfg(feature = "serde")]
#[derive(Default)]
pub struct SerdeStructAttr(StructAttr);

impl StructAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        #[cfg(feature = "serde")]
        crate::utils::parse_serde_attrs::<SerdeStructAttr>(attrs).for_each(|a| result.merge(a.0));
        Ok(result)
    }

    fn merge(&mut self, StructAttr {}: StructAttr) {}
}

#[cfg(feature = "serde")]
impl_parse! {
    SerdeStructAttr(input, out) {
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
