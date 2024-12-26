use example_core::{mount, Ctx};

mod api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let router = mount();
    let (procedures, types) = router.build().unwrap();

    // TODO: Exporting types

    tauri::Builder::default()
        .plugin(tauri_plugin_rspc::init(procedures, |_| {
            // Ctx {}
            todo!();
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
