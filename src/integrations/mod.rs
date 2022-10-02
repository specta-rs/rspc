#[cfg(any(
    feature = "httpz",
    feature = "axum",
    feature = "actix-web",
    feature = "rocket",
    feature = "lambda",
    feature = "workers"
))]
pub mod httpz;

#[cfg(feature = "tauri")]
pub mod tauri;
