use syn::Attribute;

use crate::utils::{parse_assign_str, parse_attrs};

#[derive(Default, Clone, Debug)]
pub struct FieldAttrs {
    pub id: bool,
    pub refr: bool,
    pub flatten: bool,
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a: FieldAttrs| {
            result.id = a.id || result.id;
            result.refr = a.refr || result.refr;
            result.flatten = a.flatten || result.flatten;
        });
        Ok(result)
    }
}

impl_parse! {
    FieldAttrs(input, out) {
        "id" => out.id = true,
        "refr" => out.refr = true,
        "flatten" => out.flatten = true,
    }
}

#[derive(Default, Clone, Debug)]
pub struct StructAttrs {
    pub rename: Option<String>,
}

impl StructAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a: StructAttrs| {
            result.rename = result.rename.take().or(a.rename);
        });
        Ok(result)
    }
}

impl_parse! {
    StructAttrs(input, out) {
        "rename" => out.rename = Some(parse_assign_str(input)?),
    }
}
