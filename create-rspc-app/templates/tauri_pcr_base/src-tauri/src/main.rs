#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

mod api;
mod utils;
mod prisma;

#[tokio::main]
async fn main() {
    let router = api::new().build().arced();
    let db = tauri::api::path::data_dir().unwrap().join("my_app").join("app.db");
    println!("Using database at: {:?}", db);
    let client = Arc::new(utils::load_and_migrate(db).await);

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router, move || {
            api::Ctx {
                client: Arc::clone(&client),
            }
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
