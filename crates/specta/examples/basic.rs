use specta::{ts::ts_export, Type};

#[derive(Type)]
pub struct MyCustomType {
    pub my_field: String,
}

fn main() {
    assert_eq!(
        ts_export::<MyCustomType>(),
        Ok("export interface MyCustomType { my_field: string }".to_string())
    );
}
