use crate::utils::*;

use syn::Attribute;

#[derive(Default)]
pub struct FieldAttr {
    pub skip: bool,
}

impl FieldAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a| result.merge(a));
        Ok(result)
    }

    fn merge(&mut self, FieldAttr { skip }: FieldAttr) {
        self.skip = self.skip || skip;
    }
}

impl_parse!(
    FieldAttr(input, out) {
        "skip" => out.skip = true
    }
);
