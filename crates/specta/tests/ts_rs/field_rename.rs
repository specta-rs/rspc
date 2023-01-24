#![allow(dead_code)]

use crate::ts::assert_ts;
use specta::Type;

#[derive(Type)]
struct Rename {
    a: i32,
    #[specta(rename = "bb")]
    b: i32,
}

#[test]
fn test() {
    assert_ts!(Rename, "{ a: number, bb: number }")
}
