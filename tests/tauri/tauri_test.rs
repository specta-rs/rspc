use rspc::{Config, Rspc};
use tauri::Window;

const R: Rspc<()> = Rspc::new();

#[tauri::command]
fn end_rspc_test(window: Window) {
    window.close().unwrap();
}

// Tauri must be started on the main thread so we use a custom harness for this one
fn main() {
    return; // TODO: Remove once this test is more than a syntax check

    let router = R.router().build(Config::new()).arced();

    // This requires the `OUT_DIR` env var to be set prior to running the test
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![end_rspc_test])
        .plugin(rspc::integrations::tauri::plugin(router, |window| ()))
        .setup(|app| {
            let handle = app.handle();
            std::thread::spawn(move || {
                let window = tauri::WindowBuilder::new(
                    &handle,
                    "rspc-test",
                    tauri::WindowUrl::App("index.html".into()),
                )
                .title("rspc test")
                .build()
                .unwrap();
            });
            Ok(())
        })
        .run(tauri::generate_context!("tests/tauri/tauri.config.json"))
        .expect("error while running tauri application");
}
