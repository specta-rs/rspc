use specta::{ts::export, Type};

#[derive(Type)]
#[specta(rename_all = "lowercase")]
#[specta(rename = "SimpleEnum")]
enum RenamedEnum {
    #[specta(rename = "ASDF")]
    A,
    B,
    C,
}

// #[test]
fn test_simple_enum() {
    assert_eq!(
        export::<RenamedEnum>().unwrap(),
        r#"export type SimpleEnum = "ASDF" | "b" | "c""#
    )
}
