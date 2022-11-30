#![allow(dead_code)]

use specta::{ts::inline, Type};

#[derive(Type)]
struct Rename {
    a: i32,
    #[specta(rename = "bb")]
    b: i32,
}

#[test]
fn test() {
    assert_eq!(inline::<Rename>(), "{ a: number, bb: number }")
}
