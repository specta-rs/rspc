const COMMANDS: &[&str] = &["handle_rpc"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
