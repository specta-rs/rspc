use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
enum SimpleEnum1 {
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

    assert_ts!(Empty, "never");
}

#[test]
fn test_simple_enum() {
    assert_ts!(SimpleEnum1, r#""asdf" | "B" | { C: { enumField: null } }"#)
}
