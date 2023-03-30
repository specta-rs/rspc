use std::{borrow::Cow, collections::BTreeMap};

use serde_json::Value;
use specta::{DataType, DataTypeFrom};

use crate::ExecError;

use super::{Layer, RequestContext, ValueOrStream};

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

// TODO: Remove this type once v1
pub enum EitherLayer<TCtx> {
    Legacy(Box<dyn Layer<TCtx>>),
    #[cfg(feature = "alpha")]
    Alpha(Box<dyn crate::alpha::DynLayer<TCtx>>),
}

impl<TCtx: Send + 'static> EitherLayer<TCtx> {
    pub async fn call(
        &self,
        ctx: TCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<ValueOrStream, ExecError> {
        match self {
            Self::Legacy(l) => l.call(ctx, input, req)?.into_value_or_stream().await,
            #[cfg(feature = "alpha")]
            Self::Alpha(a) => a.dyn_call(ctx, input, req)?.await,
        }
    }
}

// TODO: Make private
pub struct Procedure<TCtx> {
    pub(crate) exec: EitherLayer<TCtx>,
    pub(crate) ty: ProcedureDataType,
}

pub struct ProcedureStore<TCtx> {
    name: &'static str,
    // TODO: A `HashMap` would probs be best but due to const context's that is hard.
    pub(crate) store: BTreeMap<String, Procedure<TCtx>>,
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

        self.store.insert(
            key,
            Procedure {
                exec: EitherLayer::Legacy(exec),
                ty,
            },
        );
    }

    #[cfg(feature = "alpha")]
    pub(crate) fn append_alpha<L: crate::alpha::AlphaLayer<TCtx>>(
        &mut self,
        key: String,
        exec: L,
        ty: ProcedureDataType,
    ) where
        // TODO: move this bound to impl once `alpha` stuff is stable
        TCtx: 'static,
    {
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

        self.store.insert(
            key,
            Procedure {
                exec: EitherLayer::Alpha(exec.erase()),
                ty,
            },
        );
    }
}
