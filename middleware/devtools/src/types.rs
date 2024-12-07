use std::collections::HashMap;

use serde::Serialize;
use specta::Type;

#[derive(Clone, Serialize, Type)]
pub struct Metadata {
    pub crate_name: &'static str,
    pub crate_version: &'static str,
    pub rspc_version: &'static str,
    pub procedures: HashMap<String, ProcedureMetadata>,
}

#[derive(Clone, Serialize, Type)]
pub struct ProcedureMetadata {
    // TODO: input type
    // TOOD: output type
    // TODO: p99's
}
