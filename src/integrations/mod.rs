//! Integrations with other crates such as Axum, Tauri, etc.
//!

#[cfg(feature = "httpz")]
#[cfg_attr(docsrs, doc(cfg(feature = "httpz")))]
pub mod httpz;

#[cfg(feature = "httpz")]
#[cfg_attr(docsrs, doc(cfg(feature = "httpz")))]
pub(crate) mod httpz_extractors;

#[cfg(feature = "tauri")]
#[cfg_attr(docsrs, doc(cfg(feature = "tauri")))]
pub mod tauri;
