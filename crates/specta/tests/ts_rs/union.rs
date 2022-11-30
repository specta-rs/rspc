use specta::{
    ts::{export, inline},
    Type,
};

#[derive(Type)]
enum SimpleEnum {
    #[specta(rename = "asdf")]
    A,
    B,
    #[specta(rename_all = "camelCase")]
    C {
        enum_field: (),
    },
}

#[test]
fn test_empty() {
    #[derive(Type)]
    enum Empty {}

    assert_eq!(inline::<Empty>(), "never");
}

#[test]
fn test_simple_enum() {
    assert_eq!(
        export::<SimpleEnum>().unwrap(),
        r#"export type SimpleEnum = "asdf" | "B" | { C: { enumField: null } }"#
    )
}
