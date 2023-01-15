mod fail;
mod swift;
mod ts_rs;
mod typescript;

// TODO: Move them somewhere else
#[derive(Deserialize, Serialize, Type)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub struct SpectaTypeOverride {
    #[specta(type = String)] // Ident
    path: (),
    #[specta(type = u32)] // Ident
    path: (),
    #[specta(type = ::std::string::String)] // Path
    input: (),
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
