use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
#[specta(rename_all = "lowercase")]
#[specta(rename = "SimpleEnum")]
enum RenamedEnum {
    #[specta(rename = "ASDF")]
    A,
    B,
    C,
}

#[test]
fn test_simple_enum() {
    assert_ts!(RenamedEnum, r#""ASDF" | "b" | "c""#)
}
