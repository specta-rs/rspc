use specta::{Type, ts_definition};

#[derive(Type)]
struct Rename{
    a: i32,
    #[specta(rename = "bb")]
    b: i32
}

#[test]
fn test() {
    assert_eq!(ts_definition::<Rename>(), "{ a: number, bb: number }")
}
