use specta::{Type, ts_inline};

#[derive(Type)]
#[serde(tag = "kind", content = "d")]
enum SimpleEnum {
    A,
    B
}

#[derive(Type)]
#[serde(tag = "kind", content = "data")]
enum ComplexEnum {
    A,
    B { foo: String, bar: f64 },
    W(SimpleEnum),
    F { nested: SimpleEnum },
    T(i32, SimpleEnum)
}

#[derive(Type)]
#[serde(untagged)]
enum Untagged {
    Foo(String),
    Bar(i32),
    None
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_enum() {
    assert_eq!(
        ts_inline::<SimpleEnum>(),
        r#"{ kind: "A" } | { kind: "B" }"#
    );
    assert_eq!(
        ts_inline::<ComplexEnum>(),
        r#"{ kind: "A" } | { kind: "B", data: { foo: string, bar: number } } | { kind: "W", data: SimpleEnum } | { kind: "F", data: { nested: SimpleEnum } } | { kind: "T", data: [number, SimpleEnum] }"#
    );
    assert_eq!(
        ts_inline::<Untagged>(),
        r#"string | number | null"#
    );
}
