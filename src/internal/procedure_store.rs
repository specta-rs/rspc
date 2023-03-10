use std::collections::BTreeMap;

use specta::DataType;

use super::Layer;

// TODO: Make private
#[derive(Debug)]
pub struct ProcedureDataType {
    pub arg_ty: DataType,
    pub result_ty: DataType,
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
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            store: BTreeMap::new(),
        }
    }

    pub fn append(&mut self, key: String, exec: Box<dyn Layer<TCtx>>, ty: ProcedureDataType) {
        #[allow(clippy::panic)]
        if key.is_empty() || key == "ws" || key.starts_with("rpc.") || key.starts_with("rspc.") {
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
