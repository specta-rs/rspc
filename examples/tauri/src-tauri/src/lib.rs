use std::path::PathBuf;
use example_core::{mount, Ctx};

mod api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let router = mount();
    let (procedures, types) = router.build().unwrap();

    rspc::Typescript::default()
        // .formatter(specta_typescript::formatter::prettier)
        .header("// My custom header")
        // .enable_source_maps() // TODO: Fix this
        .export_to(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bindings.ts"),
            &types,
        )
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_rspc::init(procedures, |_| Ctx {}))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
