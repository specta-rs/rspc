use specta::{ts_inline, Type};

#[derive(Type)]
struct Skip {
    a: i32,
    b: i32,
    #[specta(skip)]
    c: String,
}

#[test]
fn test_def() {
    assert_eq!(ts_inline::<Skip>(), "{ a: number, b: number }");
}
