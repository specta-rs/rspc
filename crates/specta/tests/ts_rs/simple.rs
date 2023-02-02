use std::cell::RefCell;

use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
struct Simple {
    a: i32,
    b: String,
    c: (i32, String, RefCell<i32>),
    d: Vec<String>,
    e: Option<String>,
}

#[test]
fn test_def() {
    assert_ts!(
        Simple,
        "{ a: number; b: string; c: [number, string, number]; d: string[]; e: string | null }"
    );
}
