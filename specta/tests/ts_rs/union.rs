use specta::{Type, ts_definition, ts_export};

#[derive(Type)]
enum SimpleEnum {
    #[specta(rename = "asdf")]
    A,
    B,
    C
}

#[test]
fn test_empty() {
    #[derive(Type)]
    enum Empty {}

    assert_eq!(ts_definition::<Empty>(), "never");
}

#[test]
fn test_simple_enum() {
    assert_eq!(
        ts_export::<SimpleEnum>().unwrap(),
        r#"export type SimpleEnum = "asdf" | "B" | "C""#
    )
}
