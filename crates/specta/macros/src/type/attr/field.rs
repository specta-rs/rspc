use syn::{Result, Type, TypePath};

use crate::utils::MetaAttr;

#[derive(Default)]
pub struct FieldAttr {
    pub rename: Option<String>,
    pub r#type: Option<Type>,
    pub inline: bool,
    pub skip: bool,
    pub optional: bool,
    pub flatten: bool,
}

impl_parse! {
    FieldAttr(attr, out) {
        "rename" => out.rename = out.rename.take().or(Some(attr.pass_string()?)),
        "type" => out.r#type = out.r#type.take().or(Some(Type::Path(TypePath {
            qself: None,
            path: attr.pass_path()?,
        }))),
        "inline" => out.inline = true,
        "skip" => out.skip = true,
        "skip_serializing" => out.skip = true,
        "skip_deserializing" => out.skip = true,
        "skip_serializing_if" => out.optional = attr.pass_string()? == *"Option::is_none",
        "optional" => out.optional = true,
        "flatten" => out.flatten = true,
    }
}

impl FieldAttr {
    pub fn from_attrs(attrs: &mut Vec<MetaAttr>) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs("specta", attrs, &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs("serde", attrs, &mut result)?;
        Ok(result)
    }
}
