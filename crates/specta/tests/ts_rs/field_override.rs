use specta::{ts::ts_inline, Type};

#[derive(Type)]
struct Override {
    a: i32,
    #[specta(type = String)]
    b: i32,
}

#[test]
fn test() {
    assert_eq!(ts_inline::<Override>(), "{ a: number, b: string }")
}
