//! Integrations rspc with other crates in the ecosystem such as Axum, Tauri, etc.

#[cfg(feature = "httpz")]
#[cfg_attr(docsrs, doc(cfg(feature = "httpz")))]
pub mod httpz;

#[cfg(feature = "tauri")]
#[cfg_attr(docsrs, doc(cfg(feature = "tauri")))]
pub mod tauri;
