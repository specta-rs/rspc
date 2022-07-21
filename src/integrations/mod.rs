#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "tauri")]
pub mod tauri;

#[cfg(any(feature = "axum", feature = "tauri"))]
pub(super) mod utils;
