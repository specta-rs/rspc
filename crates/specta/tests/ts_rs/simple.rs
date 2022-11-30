use std::cell::RefCell;

use specta::{ts::inline, Type};

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
    assert_eq!(
        inline::<Simple>(),
        "{ a: number, b: string, c: [number, string, number], d: Array<string>, e: string | null }"
    )
}
