use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
#[serde(tag = "kind", content = "d")]
enum SimpleEnumA {
    A,
    B,
}

#[derive(Type)]
#[serde(tag = "kind", content = "data")]
enum ComplexEnum {
    A,
    B { foo: String, bar: f64 },
    W(SimpleEnumA),
    F { nested: SimpleEnumA },
    T(i32, SimpleEnumA),
}

#[derive(Type)]
#[serde(untagged)]
enum Untagged {
    Foo(String),
    Bar(i32),
    None,
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_enum() {
    assert_ts!(SimpleEnumA, r#"{ kind: "A" } | { kind: "B" }"#);
    assert_ts!(
        ComplexEnum,
        r#"{ kind: "A" } | { kind: "B"; data: { foo: string; bar: number } } | { kind: "W"; data: SimpleEnumA } | { kind: "F"; data: { nested: SimpleEnumA } } | { kind: "T"; data: [number, SimpleEnumA] }"#
    );
    assert_ts!(Untagged, r#"string | number | null"#);
}
