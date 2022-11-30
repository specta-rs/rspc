use syn::{Attribute, Result};

use crate::utils::*;

#[derive(Default, Clone, Debug)]
pub struct ContainerAttr {
    pub rename_all: Option<Inflection>,
    pub rename: Option<String>,
    pub tag: Option<String>,
    pub crate_name: Option<String>,
    pub inline: bool,
    pub remote: Option<String>,
}

#[cfg(feature = "serde")]
#[derive(Default)]
pub struct SerdeContainerAttr(ContainerAttr);

impl ContainerAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = Self::default();
        parse_attrs(attrs)?.for_each(|a| result.merge(a));
        #[cfg(feature = "serde")]
        crate::utils::parse_serde_attrs::<SerdeContainerAttr>(attrs)
            .for_each(|a| result.merge(a.0));
        Ok(result)
    }

    fn merge(
        &mut self,
        ContainerAttr {
            rename,
            rename_all,
            tag,
            crate_name,
            inline,
            remote,
        }: ContainerAttr,
    ) {
        self.rename = self.rename.take().or(rename);
        self.rename_all = self.rename_all.take().or(rename_all);
        self.tag = self.tag.take().or(tag);
        self.crate_name = self.crate_name.take().or(crate_name);
        self.inline = self.inline || inline;
        self.remote = self.remote.take().or(remote);
    }
}

impl_parse! {
    ContainerAttr(input, out) {
        "rename" => out.rename = Some(parse_assign_str(input)?),
        "rename_all" => out.rename_all = Some(parse_assign_inflection(input)?),
        "crate" => out.crate_name = Some(parse_assign_str(input)?),
        "inline" => out.inline = true,
        "remote" => out.remote = Some(parse_assign_str(input)?)
    }
}

#[cfg(feature = "serde")]
impl_parse! {
    SerdeContainerAttr(input, out) {
        "rename" => out.0.rename = Some(parse_assign_str(input)?),
        "rename_all" => out.0.rename_all = Some(parse_assign_inflection(input)?),
        "tag" => out.0.tag = Some(parse_assign_str(input)?),
    }
}
