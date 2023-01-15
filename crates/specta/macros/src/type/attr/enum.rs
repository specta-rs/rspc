use proc_macro2::Span;
use syn::{Error, Result};

use crate::utils::MetaAttr;

#[derive(Copy, Clone)]
pub enum Tagged<'a> {
    Externally,
    Adjacently { tag: &'a str, content: &'a str },
    Internally { tag: &'a str },
    Untagged,
}

#[derive(Default)]
pub struct EnumAttr {
    pub tag: Option<String>,
    pub content: Option<String>,
    pub untagged: bool,
}

impl_parse! {
    EnumAttr(attr, out) {
        "tag" => out.tag = out.tag.take().or(Some(attr.pass_string()?)),
        "content" => out.content = out.content.take().or(Some(attr.pass_string()?)),
        "untagged" => out.untagged = true,
    }
}

impl EnumAttr {
    pub fn from_attrs(attrs: &mut Vec<MetaAttr>) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs("specta", attrs, &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs("serde", attrs, &mut result)?;
        Ok(result)
    }

    pub fn tagged(&self) -> Result<Tagged<'_>> {
        let span = Span::call_site();
        match (self.untagged, &self.tag, &self.content) {
            (false, None, None) => Ok(Tagged::Externally),
            (false, Some(tag), None) => Ok(Tagged::Internally { tag }),
            (false, Some(tag), Some(content)) => Ok(Tagged::Adjacently { tag, content }),
            (true, None, None) => Ok(Tagged::Untagged),
            (true, Some(_), None) => Err(Error::new(span, "untagged cannot be used with tag")),
            (true, _, Some(_)) => Err(Error::new(span, "untagged cannot be used with content")),
            (false, None, Some(_)) => Err(Error::new(span, "content cannot be used without tag")),
        }
    }
}
