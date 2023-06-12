#![allow(unused_must_use)] // TODO: Remove

use rspc::Rspc;
use tauri::Window;

const R: Rspc<()> = Rspc::new();

#[tauri::command]
fn end_rspc_test(window: Window) {
    window.close().unwrap();
}

// Tauri must be started on the main thread so we use a custom harness for this one
fn main() {
    // let router = R.router().build().unwrap().arced();

    // // This requires the `OUT_DIR` env var to be set prior to running the test
    // tauri::Builder::default()
    //     .invoke_handler(tauri::generate_handler![end_rspc_test])
    //     .plugin(rspc::integrations::tauri::plugin(router, |_| ()))
    //     .setup(|app| {
    //         let handle = app.handle();
    //         std::thread::spawn(move || {
    //             tauri::WindowBuilder::new(
    //                 &handle,
    //                 "rspc-test",
    //                 tauri::WindowUrl::App("index.html".into()),
    //             )
    //             .title("rspc test")
    //             .build()
    //             .unwrap();
    //         });
    //         Ok(())
    //     });
    // // TODO: Reenable this once this test is more than a syntax check
    // // .run(tauri::generate_context!("tests/tauri/tauri.config.json"))
    // // .expect("error while running tauri application");
}
