use syn::{Attribute, Result};

#[derive(Debug, Default)]
pub struct EnumAttr {
    pub tag: Option<String>,
    pub content: Option<String>,
    pub untagged: bool,
}

#[cfg(feature = "serde")]
#[derive(Default)]
pub struct SerdeEnumAttr(EnumAttr);

#[derive(Copy, Clone)]
pub enum Tagged<'a> {
    Externally,
    Adjacently { tag: &'a str, content: &'a str },
    Internally { tag: &'a str },
    Untagged,
}

impl EnumAttr {
    pub fn tagged(&self) -> Result<Tagged<'_>> {
        match (self.untagged, &self.tag, &self.content) {
            (false, None, None) => Ok(Tagged::Externally),
            (false, Some(tag), None) => Ok(Tagged::Internally { tag }),
            (false, Some(tag), Some(content)) => Ok(Tagged::Adjacently { tag, content }),
            (true, None, None) => Ok(Tagged::Untagged),
            (true, Some(_), None) => syn_err!("untagged cannot be used with tag"),
            (true, _, Some(_)) => syn_err!("untagged cannot be used with content"),
            (false, None, Some(_)) => syn_err!("content cannot be used without tag"),
        }
    }

    #[allow(unused_variables)]
    #[allow(unused_mut)]
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        // parse_attrs(attrs)?.for_each(|a| result.merge(a));
        #[cfg(feature = "serde")]
        crate::utils::parse_serde_attrs::<SerdeEnumAttr>(attrs).for_each(|a| result.merge(a.0));
        Ok(result)
    }

    #[allow(dead_code)]
    fn merge(
        &mut self,
        EnumAttr {
            tag,
            content,
            untagged,
        }: EnumAttr,
    ) {
        self.untagged = self.untagged || untagged;
        self.tag = self.tag.take().or(tag);
        self.content = self.content.take().or(content);
    }
}

// impl_parse! {
//     EnumAttr(input, out) {
//     }
// }

#[cfg(feature = "serde")]
impl_parse! {
    SerdeEnumAttr(input, out) {
        "tag" => out.0.tag = Some(crate::utils::parse_assign_str(input)?),
        "content" => out.0.content = Some(crate::utils::parse_assign_str(input)?),
        "untagged" => out.0.untagged = true
    }
}
