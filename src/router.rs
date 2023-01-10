use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use futures::Stream;
use serde_json::Value;
use specta::{
    ts::{self, ExportConfiguration},
    DataTypeFrom, TypeDefs,
};

use crate::{
    internal::{
        GlobalData, LayerReturn, Procedure, ProcedureDataType, ProcedureKind, ProcedureStore,
        RequestContext,
    },
    Config, ExecError, ExportError,
};

/// TODO
pub struct Router<TCtx = (), TMeta = ()>
where
    TCtx: 'static,
{
    pub data: GlobalData,
    pub(crate) config: Config,
    pub(crate) queries: ProcedureStore<TCtx>,
    pub(crate) mutations: ProcedureStore<TCtx>,
    pub(crate) subscriptions: ProcedureStore<TCtx>,
    pub(crate) typ_store: TypeDefs,
    pub(crate) phantom: PhantomData<TMeta>,
}

// TODO: Move this out of this file
// TODO: Rename??
#[derive(Debug, Copy, Clone)]
pub enum ExecKind {
    Query,
    Mutation,
}

impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: 'static,
{
    pub async fn exec(
        &self,
        ctx: TCtx,
        kind: ExecKind,
        key: String,
        input: Option<Value>,
    ) -> Result<Value, ExecError> {
        let (operations, kind) = match kind {
            ExecKind::Query => (&self.queries.store, ProcedureKind::Query),
            ExecKind::Mutation => (&self.mutations.store, ProcedureKind::Mutation),
        };

        match operations
            .get(&key)
            .ok_or_else(|| ExecError::OperationNotFound(key.clone()))?
            .exec
            .call(
                ctx,
                input.unwrap_or(Value::Null),
                RequestContext {
                    kind,
                    path: key.clone(),
                },
            )?
            .into_layer_return()
            .await?
        {
            LayerReturn::Request(v) => Ok(v),
            LayerReturn::Stream(_) => Err(ExecError::UnsupportedMethod(key)),
        }
    }

    pub async fn exec_subscription(
        &self,
        ctx: TCtx,
        key: String,
        input: Option<Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>, ExecError> {
        match self
            .subscriptions
            .store
            .get(&key)
            .ok_or_else(|| ExecError::OperationNotFound(key.clone()))?
            .exec
            .call(
                ctx,
                input.unwrap_or(Value::Null),
                RequestContext {
                    kind: ProcedureKind::Subscription,
                    path: key.clone(),
                },
            )?
            .into_layer_return()
            .await?
        {
            LayerReturn::Request(_) => Err(ExecError::UnsupportedMethod(key)),
            LayerReturn::Stream(s) => Ok(s),
        }
    }

    pub fn arced(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn typ_store(&self) -> TypeDefs {
        self.typ_store.clone()
    }

    pub fn queries(&self) -> &BTreeMap<String, Procedure<TCtx>> {
        &self.queries.store
    }

    pub fn mutations(&self) -> &BTreeMap<String, Procedure<TCtx>> {
        &self.mutations.store
    }

    pub fn subscriptions(&self) -> &BTreeMap<String, Procedure<TCtx>> {
        &self.subscriptions.store
    }

    #[allow(clippy::unwrap_used)]
    pub fn export_ts<TPath: AsRef<Path>>(&self, export_path: TPath) -> Result<(), ExportError> {
        let export_path = PathBuf::from(export_path.as_ref());
        if let Some(export_dir) = export_path.parent() {
            fs::create_dir_all(export_dir)?;
        }
        let mut file = File::create(export_path)?;
        if let Some(header) = &self.config.bindings_header {
            writeln!(file, "{}", header)?;
        }
        writeln!(file, "// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.")?;

        let config = ExportConfiguration {
            bigint: ts::BigIntExportBehavior::FailWithReason(
                "rspc does not support exporting bigint types (i64, u64, i128, u128) because they are lossily decoded by `JSON.parse` on the frontend. Tracking issue: https://github.com/oscartbeaumont/rspc/issues/93",
            ),
        };

        writeln!(
            file,
            "{}",
            ts::export_datatype(&config, &Procedures::new(self).into()).unwrap()
        )?;

        for export in self
            .typ_store
            .values()
            .filter_map(|v| ts::export_datatype(&config, v).ok())
        {
            writeln!(file, "\n{}", export)?;
        }

        Ok(())
    }
}

#[derive(DataTypeFrom)]
struct Procedures {
    pub queries: Vec<ProcedureDataType>,
    pub mutations: Vec<ProcedureDataType>,
    pub subscriptions: Vec<ProcedureDataType>,
}

impl Procedures {
    pub fn new<TCtx, TMeta>(router: &Router<TCtx, TMeta>) -> Self {
        Self {
            queries: store_to_datatypes(&router.queries.store),
            mutations: store_to_datatypes(&router.mutations.store),
            subscriptions: store_to_datatypes(&router.subscriptions.store),
        }
    }
}

fn store_to_datatypes<Ctx>(
    procedures: &BTreeMap<String, Procedure<Ctx>>,
) -> Vec<ProcedureDataType> {
    procedures
        .values()
        .map(|p| p.ty.clone())
        .collect::<Vec<_>>()
}
