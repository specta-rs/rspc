use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::PathBuf,
};

use ts_rs::TS;

use crate::{KeyDefinition, MiddlewareChainBase, ResolverResult, TSDependency, TypeDef};

/// TODO
pub(crate) struct Operation<TOperationKey, TCtx>
where
    TOperationKey: KeyDefinition,
{
    name: &'static str, // TODO: move this to a const generic when support for that is added
    operations: BTreeMap<TOperationKey::KeyRaw, MiddlewareChainBase<TCtx>>,
    type_defs: BTreeMap<TOperationKey::KeyRaw, TypeDef>,
}

impl<TOperationKey, TCtx> Operation<TOperationKey, TCtx>
where
    TOperationKey: KeyDefinition,
    TCtx: 'static,
{
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            operations: BTreeMap::new(),
            type_defs: BTreeMap::new(),
        }
    }

    pub fn insert<TArg: TS, TResolverMarker, TResolverResult: ResolverResult<TResolverMarker>>(
        &mut self,
        key: TOperationKey::KeyRaw,
        handler: MiddlewareChainBase<TCtx>,
    ) {
        if self.operations.contains_key(&key) {
            panic!(
                "rspc error: operation '{}' already has resolver with name {:?}",
                self.name, key
            );
        }

        self.operations.insert(key.clone(), Box::new(handler));
        self.type_defs
            .insert(key, TResolverResult::type_def::<TArg>());
    }

    pub(crate) fn insert_internal(
        &mut self,
        key: TOperationKey::KeyRaw,
        handler: MiddlewareChainBase<TCtx>,
    ) {
        if self.operations.contains_key(&key) {
            panic!(
                "rspc error: operation '{}' already has resolver with name {:?}",
                self.name, key
            );
        }

        self.operations.insert(key, Box::new(handler));
    }

    pub(crate) fn insert_typedefs(&mut self, type_defs: BTreeMap<TOperationKey::KeyRaw, TypeDef>) {
        self.type_defs.extend(type_defs);
    }

    pub fn get(&self, key: TOperationKey::KeyRaw) -> Option<&MiddlewareChainBase<TCtx>> {
        self.operations.get(&key)
    }

    pub(crate) fn consume(
        self,
    ) -> (
        BTreeMap<TOperationKey::KeyRaw, MiddlewareChainBase<TCtx>>,
        BTreeMap<TOperationKey::KeyRaw, TypeDef>,
    ) {
        (self.operations, self.type_defs)
    }

    // TODO: Don't use `Box<Error>` as return type.
    pub(crate) fn export(
        &self,
        dependencies: &mut BTreeSet<TSDependency>,
        buf: &mut Vec<u8>,
        export_path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.type_defs.len() == 0 {
            write!(buf, "never")?;
        }

        for (key, type_def) in self.type_defs.iter() {
            // TODO: Handle errors
            (type_def.arg_export)(export_path.join(format!("{}.ts", type_def.arg_ty_name)));
            (type_def.result_export)(export_path.join(format!("{}.ts", type_def.result_ty_name)));

            dependencies.extend(type_def.dependencies.clone());

            write!(
                buf,
                " | {{ key: \"{}\"; arg: {}; result: {}; }}",
                key.to_string(),
                type_def.arg_ty_name,
                type_def.result_ty_name
            )?;
        }

        Ok(())
    }
}
