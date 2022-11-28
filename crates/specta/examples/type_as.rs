use specta::{ts::ts_export, Type};

pub enum MyEnum {
    A,
    B,
    C,
}

#[derive(Type)]
pub struct MyCustomType {
    #[specta(type_as=String)]
    pub nested: MyEnum,
}

fn main() {
    println!("{}", ts_export::<MyCustomType>().unwrap(),);
}
