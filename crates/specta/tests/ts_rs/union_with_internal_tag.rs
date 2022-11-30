use specta::{ts::inline, Type};

#[derive(Type)]
#[serde(tag = "type")]
enum EnumWithInternalTag {
    A { foo: String },
    B { bar: i32 },
}

#[derive(Type)]
struct InnerA {
    foo: String,
}

#[derive(Type)]
struct InnerB {
    bar: i32,
}

#[derive(Type)]
#[serde(tag = "type")]
enum EnumWithInternalTag2 {
    A(InnerA),
    B(InnerB),
}

#[test]
#[cfg(feature = "serde")]
fn test_enums_with_internal_tags() {
    assert_eq!(
        inline::<EnumWithInternalTag>(),
        r#"{ type: "A", foo: string } | { type: "B", bar: number }"#
    );

    assert_eq!(
        inline::<EnumWithInternalTag2>(),
        r#"{ type: "A" } & InnerA | { type: "B" } & InnerB"#
    );
}
