use specta::{ts::inline, Type};

#[test]
fn rename_all() {
    #[derive(Type)]
    #[specta(rename_all = "UPPERCASE")]
    struct Rename {
        a: i32,
        b: i32,
    }

    assert_eq!(inline::<Rename>(), "{ A: number, B: number }");
}

#[cfg(feature = "serde")]
#[test]
fn serde_rename_special_char() {
    #[derive(serde::Serialize, Type)]
    struct RenameSerdeSpecialChar {
        #[serde(rename = "a/b")]
        b: i32,
    }

    assert_eq!(inline::<RenameSerdeSpecialChar>(), r#"{ "a/b": number }"#);
}
