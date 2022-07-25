use specta::{ts_definition, Type};

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
        ts_definition::<TaggedType>(),
        "{ type: \"TaggedType\", a: number, b: number }"
    )
}

#[test]
#[cfg(not(feature = "serde"))]
fn test() {
    assert_eq!(ts_definition::<TaggedType>(), "{ a: number, b: number }")
}
