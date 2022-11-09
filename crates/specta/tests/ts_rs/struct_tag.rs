use specta::{ts_inline, Type};

#[derive(Type)]
#[serde(tag = "type")]
struct TaggedType {
    a: i32,
    b: i32,
}

#[test]
#[cfg(feature = "serde")]
fn test() {
    assert_eq!(
        ts_inline::<TaggedType>(),
        "{ a: number, b: number, type: \"TaggedType\" }"
    )
}

#[test]
#[cfg(not(feature = "serde"))]
fn test() {
    assert_eq!(ts_inline::<TaggedType>(), "{ a: number, b: number }")
}
