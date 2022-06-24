use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use ts_rs::{Dependency, ExportError, TS};

use crate::{Context, Resolver, ResolverResult};

pub struct Router<TCtx: 'static> {
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
}

impl<TCtx> Router<TCtx> {
    pub fn new() -> Self {
        Self {
            query: BTreeMap::new(),
            query_types: BTreeMap::new(),
            mutation: BTreeMap::new(),
            mutation_types: BTreeMap::new(),
        }
    }

    pub fn query<
        TType,
        TArgs: DeserializeOwned + TS + 'static,
        TResolver: Resolver<TType> + 'static,
    >(
        mut self,
        name: &'static str,
        resolver: fn(Context<TCtx>, TArgs) -> TResolver,
    ) -> Self {
        if self.query.contains_key(name) {
            panic!("trpc-rs error: query with name '{}' already exists", name);
        }

        self.query.insert(
            name,
            Box::new(move |ctx, args| {
                resolver(ctx, serde_json::from_value(args).unwrap()).resolve()
            }),
        );
        self.query_types.insert(
            name,
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
        TType,
        TArgs: DeserializeOwned + TS + 'static,
        TResolver: Resolver<TType> + 'static,
    >(
        mut self,
        name: &'static str,
        resolver: fn(Context<TCtx>, TArgs) -> TResolver,
    ) -> Self {
        if self.mutation.contains_key(name) {
            panic!("trpc-rs error: query with name '{}' already exists", name);
        }

        self.mutation.insert(
            name,
            Box::new(move |ctx, args| {
                resolver(ctx, serde_json::from_value(args).unwrap()).resolve()
            }),
        );
        self.mutation_types.insert(
            name,
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

    pub async fn exec_query<S: AsRef<str>, TArgs: Serialize>(
        &self,
        ctx: TCtx,
        name: S,
        args: TArgs,
    ) -> Result<Value, ()> {
        let name = name.as_ref();
        let result =
            self.query.get(name).ok_or(())?(Context { ctx }, serde_json::to_value(args).unwrap());

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

    pub async fn exec_mutation<S: AsRef<str>, TArgs: Serialize>(
        &self,
        ctx: TCtx,
        name: S,
        args: TArgs,
    ) -> Result<Value, ()> {
        let name = name.as_ref();
        let result = self.mutation.get(name).ok_or(())?(
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

    use super::*;

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
}
