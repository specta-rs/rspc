mod fail;
mod ts_rs;
mod typescript;

#[test]
fn export_types() {
    let types = specta::export::TYPES.lock().unwrap();

    dbg!(types);
}
