use specta::Type;

use crate::ts::assert_ts;

// This test is to do with how the Macro passes the tokens
#[derive(Type)]
pub struct SpectaTypeOverride {
    #[specta(type = String)] // Ident
    string_ident: (),
    #[specta(type = u32)] // Ident
    u32_ident: (),
    #[specta(type = ::std::string::String)] // Path
    path: (),
}

// Checking that you can override the type of a field that is invalid. This is to ensure user code can override Specta in the case we have a bug/unsupported type.
#[derive(Type)]
pub struct InvalidToValidType {
    #[specta(type = Option<serde_json::Value>)]
    pub(crate) cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[test]
fn type_override() {
    assert_ts!(
        SpectaTypeOverride,
        "{ string_ident: string; u32_ident: number; path: string }"
    );
    assert_ts!(InvalidToValidType, "{ cause: any | null }");
}
