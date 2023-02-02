use serde::Serialize;
use specta::Type;

use crate::ts::assert_ts;

#[derive(Type, Serialize)]
struct Bar {
    field: i32,
}

#[derive(Type, Serialize)]
struct Foo {
    bar: Bar,
}

#[derive(Type, Serialize)]
enum SimpleEnum2 {
    A(String),
    B(i32),
    C,
    D(String, i32),
    E(Foo),
    F { a: i32, b: String },
}

#[test]
fn test_stateful_enum() {
    assert_ts!(Bar, r#"{ field: number }"#);

    assert_ts!(Foo, r#"{ bar: Bar }"#);

    assert_ts!(
        SimpleEnum2,
        r#"{ A: string } | { B: number } | "C" | { D: [string, number] } | { E: Foo } | { F: { a: number; b: string } }"#
    );
}
