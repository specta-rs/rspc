use specta::{ts::ts_inline, Type};

#[derive(Type)]
struct Rename {
    a: i32,
    #[specta(rename = "bb")]
    b: i32,
}

#[test]
fn test() {
    assert_eq!(ts_inline::<Rename>(), "{ a: number, bb: number }")
}
