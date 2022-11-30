use specta::{ts, Type};

pub enum MyEnum {
    A,
    B,
    C,
}

#[derive(Type)]
pub struct MyCustomType {
    #[specta(type = String)]
    pub nested: MyEnum,
}

fn main() {
    println!("{}", ts::export::<MyCustomType>().unwrap(),);
}
