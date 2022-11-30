use std::collections::BTreeMap;

use specta::{datatype::DataType, DataTypeFrom};

use crate::is_valid_procedure_name;

use super::Layer;

#[derive(Debug, Clone, DataTypeFrom)]
pub struct ProcedureDataType {
    pub key: String,
    pub input: DataType,
    pub result: DataType,
    /// TODO: Remove these
    #[specta(skip)]
    pub inline_input: DataType,
    #[specta(skip)]
    pub inline_result: DataType,
}

// TODO: Make private
pub struct Procedure<TCtx> {
    pub exec: Box<dyn Layer<TCtx>>,
    pub ty: ProcedureDataType,
}

pub struct ProcedureStore<TCtx> {
    name: &'static str,
    pub store: BTreeMap<String, Procedure<TCtx>>,
}

impl<TCtx> ProcedureStore<TCtx> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            store: Default::default(),
        }
    }

    pub fn append(&mut self, key: String, exec: Box<dyn Layer<TCtx>>, ty: ProcedureDataType) {
        #[allow(clippy::panic)]
        if is_valid_procedure_name(&key) {
            panic!(
                "rspc error: attempted to create {} operation named '{}', however this name is not allowed.",
                self.name,
                key
            );
        }

        #[allow(clippy::panic)]
        if self.store.contains_key(&key) {
            panic!(
                "rspc error: {} operation already has resolver with name '{}'",
                self.name, key
            );
        }

        self.store.insert(key, Procedure { exec, ty });
    }
}
