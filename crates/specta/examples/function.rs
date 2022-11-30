use specta::*;

#[specta]
#[allow(unused)]
fn some_function(name: String, age: i32) -> bool {
    true
}

fn main() {
    // This API is pretty new and will likely undergo API changes in the future.
    assert_eq!(
        ts::export_datatype(&fn_datatype!(some_function).into()),
        Ok("export interface CommandDataType { name: \"some_function\", input: { name: string, age: number }, result: boolean }".to_string())
    );
}
