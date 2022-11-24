use specta::{ts::ts_export, Type};

#[derive(Type)]
pub struct MyCustomType<A> {
    pub my_field: String,
    pub generic: A,
}

fn main() {
    dbg!(MyCustomType::<()>::definition_generics());

    assert_eq!(
        ts_export::<MyCustomType<()>>(),
        Ok("export interface MyCustomType<A> { my_field: string, generic: A }".to_string())
    );
}
