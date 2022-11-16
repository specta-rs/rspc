use crate::utils::*;

use syn::{Attribute, Result};

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

#[derive(Default, Clone, Debug)]
pub struct ContainerAttr {
    pub crate_name: Option<String>,
}

impl ContainerAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a| result.merge(a));
        Ok(result)
    }

    fn merge(&mut self, ContainerAttr { crate_name }: ContainerAttr) {
        self.crate_name = self.crate_name.take().or(crate_name);
    }
}

impl_parse! {
    ContainerAttr(input, out) {
        "crate" => out.crate_name = Some(parse_assign_str(input)?),
    }
}
