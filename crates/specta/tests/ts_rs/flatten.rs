#![allow(dead_code)]

use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
struct A {
    a: i32,
    b: i32,
}

#[derive(Type)]
struct B {
    #[specta(flatten)]
    a: A,
    c: i32,
}

#[derive(Type)]
struct C {
    #[specta(inline)]
    b: B,
    d: i32,
}

#[test]
fn test() {
    assert_ts!(
        C,
        "{ b: ({ a: number; b: number }) & { c: number }; d: number }"
    )
}
