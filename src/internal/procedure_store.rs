mod private {
    use std::{borrow::Cow, collections::BTreeMap};

    use specta::{DataType, DataTypeFrom};

    use crate::internal::{DynLayer, Layer};

    /// Represents a Typescript procedure file which is generated by the Rust code.
    /// This is codegenerated Typescript file is how we can validate the types on the frontend match Rust.
    ///
    /// @internal
    #[derive(Debug, Clone, DataTypeFrom)]
    #[cfg_attr(test, derive(specta::Type))]
    #[cfg_attr(test, specta(rename = "ProcedureDef"))]
    pub struct ProcedureDataType {
        pub key: Cow<'static, str>,
        #[specta(type = serde_json::Value)]
        pub input: DataType,
        #[specta(type = serde_json::Value)]
        pub result: DataType,
    }

    // TODO: Rename this
    pub struct ProcedureTodo<TCtx> {
        pub(crate) exec: Box<dyn DynLayer<TCtx>>,
        pub(crate) ty: ProcedureDataType,
    }

    pub struct ProcedureStore<TCtx> {
        pub(crate) name: &'static str,
        pub(crate) store: BTreeMap<String, ProcedureTodo<TCtx>>,
    }

    impl<TCtx: 'static> ProcedureStore<TCtx> {
        pub const fn new(name: &'static str) -> Self {
            Self {
                name,
                store: BTreeMap::new(),
            }
        }

        pub fn append<L: Layer<TCtx>>(&mut self, key: String, exec: L, ty: ProcedureDataType) {
            // TODO: Cleanup this logic and do better router merging
            #[allow(clippy::panic)]
            if key.is_empty() || key == "ws" || key.starts_with("rpc.") || key.starts_with("rspc.")
            {
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

            self.store.insert(
                key,
                ProcedureTodo {
                    exec: exec.erase(),
                    ty,
                },
            );
        }
    }
}

use crate::BuildErrorCause;

pub(crate) fn is_valid_name(name: &str) -> Option<BuildErrorCause> {
    if name.len() > 255 {
        return Some(BuildErrorCause::InvalidName);
    }

    for c in name.chars() {
        if !(c.is_alphanumeric() || c == '_' || c == '-' || c == '~') {
            return Some(BuildErrorCause::InvalidCharInName(c));
        }
    }

    return None;
}

pub(crate) use private::{ProcedureDataType, ProcedureStore, ProcedureTodo};
