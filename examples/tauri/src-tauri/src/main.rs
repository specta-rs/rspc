// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rspc::Rspc;

const R: Rspc<()> = Rspc::new();

#[tokio::main]
async fn main() {
    let router = R.router().build().unwrap().arced();

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router, |_| ()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
