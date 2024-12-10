use api::Infallible;
use rspc::{Procedure2, Router2};

mod api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: Show proper setup

    let router = Router2::new().procedure(
        "query",
        Procedure2::builder().query(|_, _: ()| async { Ok::<(), Infallible>(()) }),
    );
    let (procedures, types) = router.build().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_rspc::init(procedures, |_| {}))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
