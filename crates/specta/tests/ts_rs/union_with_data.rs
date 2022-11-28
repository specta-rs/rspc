use serde::Serialize;
use specta::{ts::ts_export, Type};

#[derive(Type, Serialize)]
struct Bar {
    field: i32,
}

#[derive(Type, Serialize)]
struct Foo {
    bar: Bar,
}

#[derive(Type, Serialize)]
enum SimpleEnum {
    A(String),
    B(i32),
    C,
    D(String, i32),
    E(Foo),
    F { a: i32, b: String },
}

#[test]
fn test_stateful_enum() {
    assert_eq!(
        ts_export::<Bar>().unwrap(),
        r#"export type Bar = { field: number }"#
    );

    assert_eq!(
        ts_export::<Foo>().unwrap(),
        r#"export type Foo = { bar: Bar }"#
    );

    assert_eq!(
        ts_export::<SimpleEnum>().unwrap(),
        r#"export type SimpleEnum = { A: string } | { B: number } | "C" | { D: [string, number] } | { E: Foo } | { F: { a: number, b: string } }"#
    );
}
