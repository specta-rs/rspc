use specta::{Type, ts_inline, ts_export};
use serde::Serialize;

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
    assert_eq!(ts_export::<Bar>().unwrap(), r#"export interface Bar { field: number }"#);
    // assert_eq!(Bar::dependencies(), vec![]);

    assert_eq!(ts_export::<Foo>().unwrap(), r#"export interface Foo { bar: Bar }"#);
    // assert_eq!(
    //     Foo::dependencies(),
    //     vec![Dependency::from_ty::<Bar>().unwrap()]
    // );

    assert_eq!(
        ts_export::<SimpleEnum>().unwrap(),
        r#"export type SimpleEnum = { A: string } | { B: number } | "C" | { D: [string, number] } | { E: Foo } | { F: { a: number, b: string } }"#
    );
    // assert!(SimpleEnum::dependencies()
    //     .into_iter()
    //     .all(|dep| dep == Dependency::from_ty::<Foo>().unwrap()),);
}
