use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
struct Optional {
    #[specta(optional)]
    a: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<String>,
}

#[test]
fn test() {
    // TODO: Make it sure this test is run in CI. Won't run without `--all-features`.
    #[cfg(not(feature = "serde"))]
    assert_ts!(Optional, "{ a?: number; b: string | null }");

    #[cfg(feature = "serde")]
    assert_ts!(Optional, "{ a?: number; b?: string }");
}
