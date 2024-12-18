use example_axum::{create_router, Ctx};

mod api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let router = create_router();
    let (procedures, types) = router.build().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_rspc::init(procedures, |_| Ctx {}))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
