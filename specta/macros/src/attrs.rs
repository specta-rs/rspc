use darling::{FromDeriveInput, FromField, FromVariant};

#[derive(FromDeriveInput)]
#[darling(attributes(specta))]
pub struct DeriveContainerAttrs {
    pub rename: Option<String>,
    pub rename_all: Option<String>,
    #[darling(rename = "crate")]
    pub crate_name: Option<String>,
    #[darling(default)]
    pub inline: bool
}

#[derive(FromField)]
#[darling(attributes(specta))]
pub struct DeriveStructFieldAttrs {
    pub rename: Option<String>,
    #[darling(default)]
    pub inline: bool,
    #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub optional: bool,
    #[darling(default)]
    pub flatten: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(specta))]
pub struct DeriveEnumAttrs {
    pub tag: Option<String>,
    pub content: Option<String>,
    #[darling(default)]
    pub untagged: bool,
}

#[derive(FromVariant)]
#[darling(attributes(specta))]
pub struct DeriveEnumVariantAttrs {
    pub rename: Option<String>,
    #[darling(default)]
    pub skip: bool
}
