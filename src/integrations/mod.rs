#[cfg(any(
    feature = "httpz",
    feature = "axum",
    feature = "actix-web",
    feature = "rocket",
    feature = "lambda"
))]
pub mod httpz;

#[cfg(feature = "tauri")]
pub mod tauri;
