use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use ts_rs::{Dependency, ExportError, TS};

use crate::{Context, Key, KeyDefinition, Resolver, ResolverResult};

pub struct Router<
    TCtx: 'static,
    TQueryKey: KeyDefinition = &'static str,
    TMutationKey: KeyDefinition = &'static str,
> {
    query: BTreeMap<&'static str, Box<dyn Fn(Context<TCtx>, Value) -> ResolverResult>>,
    query_types: BTreeMap<
        &'static str,
        (
            fn(PathBuf) -> Result<(String, Vec<Dependency>), ExportError>,
            fn(PathBuf) -> Result<(String, Vec<Dependency>), ExportError>,
        ),
    >,
    mutation: BTreeMap<&'static str, Box<dyn Fn(Context<TCtx>, Value) -> ResolverResult>>,
    mutation_types: BTreeMap<
        &'static str,
        (
            fn(PathBuf) -> Result<(String, Vec<Dependency>), ExportError>,
            fn(PathBuf) -> Result<(String, Vec<Dependency>), ExportError>,
        ),
    >,
    phantom: PhantomData<(TQueryKey, TMutationKey)>,
}

impl<TCtx, TQueryKey: KeyDefinition, TMutationKey: KeyDefinition>
    Router<TCtx, TQueryKey, TMutationKey>
{
    pub fn new() -> Self {
        Self {
            query: BTreeMap::new(),
            query_types: BTreeMap::new(),
            mutation: BTreeMap::new(),
            mutation_types: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn query<
        TMarker,
        TKey: Key<TQueryKey::Key, TArgs>,
        TArgs: DeserializeOwned + TS + 'static,
        TResolver: Resolver<TMarker> + 'static,
    >(
        mut self,
        key: TKey,
        resolver: fn(Context<TCtx>, TArgs) -> TResolver,
    ) -> Self {
        let key = key.to_val();
        if self.query.contains_key(key) {
            panic!("trpc-rs error: query with name '{}' already exists", key);
        }

        self.query.insert(
            key,
            Box::new(move |ctx, args| {
                resolver(ctx, serde_json::from_value(args).unwrap()).resolve()
            }),
        );
        self.query_types.insert(
            key,
            (
                |export_path| TResolver::export(export_path),
                |export_path| {
                    // TODO: This is a very suboptiomal solution for https://github.com/Aleph-Alpha/ts-rs/issues/70
                    let type_name = match <TArgs as TS>::transparent() {
                        true => <TArgs as TS>::inline(),
                        false => <TArgs as TS>::name(),
                    };

                    match <TArgs as TS>::export_to(
                        export_path.join(format!("{}.ts", <TArgs as TS>::name())),
                    ) {
                        Ok(_) | Err(ExportError::CannotBeExported) => {
                            Ok((type_name, <TArgs as TS>::dependencies()))
                        }
                        Err(v) => Err(v),
                    }
                },
            ),
        );
        self
    }

    pub fn mutation<
        TMarker,
        TKey: Key<TMutationKey::Key, TArgs>,
        TArgs: DeserializeOwned + TS + 'static,
        TResolver: Resolver<TMarker> + 'static,
    >(
        mut self,
        key: TKey,
        resolver: fn(Context<TCtx>, TArgs) -> TResolver,
    ) -> Self {
        let key = key.to_val();
        if self.mutation.contains_key(key) {
            panic!("trpc-rs error: query with name '{}' already exists", key);
        }

        self.mutation.insert(
            key,
            Box::new(move |ctx, args| {
                resolver(ctx, serde_json::from_value(args).unwrap()).resolve()
            }),
        );
        self.mutation_types.insert(
            key,
            (
                |export_path| TResolver::export(export_path),
                |export_path| {
                    // TODO: This is a very suboptiomal solution for https://github.com/Aleph-Alpha/ts-rs/issues/70
                    let type_name = match <TArgs as TS>::transparent() {
                        true => <TArgs as TS>::inline(),
                        false => <TArgs as TS>::name(),
                    };

                    match <TArgs as TS>::export_to(
                        export_path.join(format!("{}.ts", <TArgs as TS>::name())),
                    ) {
                        Ok(_) | Err(ExportError::CannotBeExported) => {
                            let mut deps = <TArgs as TS>::dependencies();
                            // This is kinda hacky. Investigate a better solution
                            if let Some(_) = <TArgs as TS>::EXPORT_TO {
                                deps.push(Dependency {
                                    type_id: TypeId::of::<TArgs>(),
                                    ts_name: <TArgs as TS>::name(),
                                    exported_to: "",
                                });
                            }
                            Ok((type_name, deps))
                        }
                        Err(v) => Err(v),
                    }
                },
            ),
        );
        self
    }

    pub fn export<TPath: AsRef<Path>>(&self, export_path: TPath) -> Result<(), ExportError> {
        let export_path = PathBuf::from(export_path.as_ref());
        fs::create_dir_all(&export_path)?;
        let mut file = File::create(export_path.clone().join("index.ts"))?;
        writeln!(file, "// This file was generated by [trpc-rs](https://github.com/oscartbeaumont/trpc-rs). Do not edit this file manually.")?;

        let mut buf = Vec::new();
        let mut dependencies = HashSet::new();
        for (field_name, (output_def, arg_def)) in self.query_types.iter() {
            let (export_ty, ty_deps) = output_def(export_path.clone())?;
            for dep in ty_deps {
                dependencies.insert(dep.ts_name);
            }

            let (arg_ty, ty_deps) = arg_def(export_path.clone())?;
            for dep in ty_deps {
                dependencies.insert(dep.ts_name);
            }

            write!(
                buf,
                "{}: {{ args: {}, output: {} }}, ",
                field_name, arg_ty, export_ty
            )?;
        }

        let mut buf = Vec::new();
        for (field_name, (output_def, arg_def)) in self.mutation_types.iter() {
            let (export_ty, ty_deps) = output_def(export_path.clone())?;
            for dep in ty_deps {
                dependencies.insert(dep.ts_name);
            }

            let (arg_ty, ty_deps) = arg_def(export_path.clone())?;
            for dep in ty_deps {
                dependencies.insert(dep.ts_name);
            }

            write!(
                buf,
                "{}: {{ args: {}, output: {} }}, ",
                field_name, arg_ty, export_ty
            )?;
        }

        for dep_name in dependencies.iter() {
            writeln!(
                file,
                "import type {{ {} }} from {:?};",
                dep_name.clone(),
                format!("./{}", dep_name)
            )?;
        }

        write!(file, "\nexport interface Query {{ ")?;
        file.write_all(&buf)?;
        writeln!(file, " }}")?;

        write!(file, "\nexport interface Mutation {{ ")?;
        file.write_all(&buf)?;
        writeln!(file, " }}")?;

        Ok(())
    }

    pub async fn exec_query<TArgs: Serialize, TKey: Key<TQueryKey::Key, TArgs>>(
        &self,
        ctx: TCtx,
        key: TKey,
        args: TArgs,
    ) -> Result<Value, ()> {
        let result = self.query.get(key.to_val()).ok_or(())?(
            Context { ctx },
            serde_json::to_value(args).unwrap(),
        );

        // TODO: Cleanup this up to support recursive resolving

        let result = match result {
            ResolverResult::Future(fut) => fut.await,
            result => result,
        };

        match result {
            ResolverResult::Value(value) => Ok(value),
            ResolverResult::Future(_) => unimplemented!(),
        }
    }

    pub async fn exec_mutation<TArgs: Serialize, TKey: Key<TMutationKey::Key, TArgs>>(
        &self,
        ctx: TCtx,
        key: TKey,
        args: TArgs,
    ) -> Result<Value, ()> {
        let result = self.mutation.get(key.to_val()).ok_or(())?(
            Context { ctx },
            serde_json::to_value(args).unwrap(),
        );

        // TODO: Cleanup this up to support recursive resolving

        let result = match result {
            ResolverResult::Future(fut) => fut.await,
            result => result,
        };

        match result {
            ResolverResult::Value(value) => Ok(value),
            ResolverResult::Future(_) => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    // use super as trpc_rs;
    use super::*;
    use trpc_rs_macros::Key;

    mod trpc_rs {
        pub use super::*;
    }

    #[derive(TS, Deserialize, Serialize)]
    pub struct MyArgs {
        pub val: String,
    }

    #[tokio::test]
    async fn basic_test() {
        let router = Router::<()>::new()
            .query("string", |_, v: String| v)
            .query("number", |_, v: i32| v)
            .query("bool", |_, v: bool| v)
            .query("customStruct", |_, v: MyArgs| v.val)
            .query("tupleString", |_, v: (String, String)| v);

        let result = router
            .exec_query((), "string", json!("Hello"))
            .await
            .unwrap();
        assert_eq!(result, json!("Hello"));

        let result = router.exec_query((), "number", json!(43i32)).await.unwrap();
        assert_eq!(result, json!(43i32));

        let result = router.exec_query((), "bool", json!(true)).await.unwrap();
        assert_eq!(result, json!(true));

        let result = router
            .exec_query(
                (),
                "customStruct",
                MyArgs {
                    val: "Testing".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(result, json!("Testing"));

        let result = router
            .exec_query((), "tupleString", json!(("Hello", "World")))
            .await
            .unwrap();
        assert_eq!(result, json!(["Hello", "World"]));
    }

    #[tokio::test]
    async fn enum_key() {
        #[derive(Key, Clone, PartialOrd, Ord, PartialEq, Eq)]
        pub enum QueryKey {
            DemoQuery,
        }

        #[derive(Key, Clone, PartialOrd, Ord, PartialEq, Eq)]
        pub enum MutationKey {
            DemoMutation,
        }

        let router = Router::<(), QueryKey, MutationKey>::new()
            .query(QueryKey::DemoQuery, |_, _: ()| "query")
            .mutation(MutationKey::DemoMutation, |_, _: ()| "mutation");

        let result = router
            .exec_query((), QueryKey::DemoQuery, json!(null))
            .await
            .unwrap();
        assert_eq!(result, json!("query"));

        let result = router
            .exec_mutation((), MutationKey::DemoMutation, json!(null))
            .await
            .unwrap();
        assert_eq!(result, json!("mutation"));
    }
}
