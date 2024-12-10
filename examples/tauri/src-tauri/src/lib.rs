use rspc::{Infallible, Procedure2, Router2};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let router = Router2::new().procedure(
        "query",
        Procedure2::builder().query(|_, _: ()| async { Ok::<(), Infallible>(()) }),
    );
    let (procedures, types) = router.build().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_rspc::init(procedures, |_| {}))
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
