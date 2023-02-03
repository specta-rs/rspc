use specta::Type;

use crate::ts::assert_ts;

#[allow(non_camel_case_types)]
#[derive(Type)]
struct r#enum {
    r#type: i32,
    r#use: i32,
    r#struct: i32,
    r#let: i32,
    r#enum: i32,
}

#[test]
fn raw_idents() {
    assert_ts!(
        r#enum,
        "{ type: number; use: number; struct: number; let: number; enum: number }"
    );
}
