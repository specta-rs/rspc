use specta::{export_fn, ts::ts_export_datatype, ToDataType};

#[specta::command]
#[allow(unused)]
fn some_function(name: String, age: i32) -> bool {
    true
}

fn main() {
    // This API is pretty new and will likely undergo API changes in the future.
    assert_eq!(
        ts_export_datatype(&export_fn!(some_function).to_data_type()),
        Ok("export interface CommandDataType { name: \"some_function\", input: { name: string, age: number }, result: boolean }".to_string())
    );
}
