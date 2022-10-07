#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

mod api;
mod utils;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");

    pretty_env_logger::init();

    let router = api::new().build().arced();
    let data_dir = tauri::api::path::data_dir().unwrap();
    let db = data_dir.join("my_app").join("app.db");
    log::info!("Using database at: {:?}", db);
    let client = Arc::new(utils::load_and_migrate(db).await);

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router, move || {
            api::Ctx {
                client: client.clone(),
            }
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
