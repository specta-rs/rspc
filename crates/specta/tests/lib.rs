mod fail;
mod swift;
mod ts;
mod ts_rs;

use serde::*;
use specta::*;
pub(crate) use ts::*;

#[derive(Deserialize, Serialize, Type)]
pub struct SpectaTypeOverride {
    #[specta(type = String)] // Ident
    string_ident: (),
    #[specta(type = u32)] // Ident
    u32_ident: (),
    #[specta(type = ::std::string::String)] // Path
    path: (),
}

#[test]
fn type_override() {
    let datatype = SpectaTypeOverride::inline(
        DefOpts {
            parent_inline: false,
            type_map: &mut Default::default(),
        },
        &[],
    );
}

// TODO: Compile Error
// #[derive(Deserialize, Serialize, Type)]
// #[serde(rename_all = "camelCase123")]
// pub enum Demo2 {}

// TODO: Compile Error
// #[derive(Debug, Clone, specta::Type)]
// pub struct Error {
//     pub(crate) cause: Option<Arc<dyn std::error::Error + Send + Sync>>,
// }

// #[derive(Debug, Clone, specta::Type)]
// pub struct Error {
//      #[specta(type = Option<serde_json::Value>)]
//     pub(crate) cause: Option<Arc<dyn std::error::Error + Send + Sync>>,
// }

// #[derive(DataTypeFrom)]
// #[cfg_attr(test, derive(specta::Type))] // This derive bit gets passed into the macro
// #[cfg_attr(test, specta(rename = "ProceduresDef"))]
// pub(crate) struct Procedures {
//     pub queries: Vec<ProcedureDataType>,
//     pub mutations: Vec<ProcedureDataType>,
//     pub subscriptions: Vec<ProcedureDataType>,
// }
