use specta::{ts::inline, Type};

#[derive(Type)]
struct Skip {
    a: i32,
    b: i32,
    #[specta(skip)]
    c: String,
}

#[test]
fn test_def() {
    assert_eq!(inline::<Skip>(), "{ a: number, b: number }");
}
