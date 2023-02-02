use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
#[serde(tag = "type")]
struct TaggedType {
    a: i32,
    b: i32,
}

#[test]
#[cfg(feature = "serde")]
fn test() {
    assert_ts!(
        TaggedType,
        r#"{ a: number; b: number; type: "TaggedType" }"#
    );
}

// TODO: Make it sure this test is run in CI. Won't run without `--all-features`.
#[test]
#[cfg(not(feature = "serde"))]
fn test() {
    assert_ts!(TaggedType, "{ a: number; b: number }");
}
