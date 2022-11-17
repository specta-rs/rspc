use specta::{ts::ts_inline, Type};

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
    assert_eq!(
        ts_inline::<r#enum>(),
        "{ type: number, use: number, struct: number, let: number, enum: number }"
    )
}
