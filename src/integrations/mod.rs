#[cfg(any(
    feature = "httpz",
    feature = "axum",
    feature = "actix-web",
    feature = "rocket",
    feature = "lambda",
    feature = "workers"
))]
pub mod httpz;

#[cfg(any(
    feature = "httpz",
    feature = "axum",
    feature = "actix-web",
    feature = "rocket",
    feature = "lambda",
    feature = "workers"
))]
pub(crate) mod httpz_extractors;

#[cfg(feature = "tauri")]
pub mod tauri;
