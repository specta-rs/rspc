use std::collections::BTreeMap;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{Context, Resolver, ResolverResult};

pub struct Router<TCtx: 'static> {
    query: BTreeMap<&'static str, Box<dyn Fn(Context<TCtx>, Value) -> ResolverResult>>,
    mutation: BTreeMap<&'static str, Box<dyn Fn(Context<TCtx>, Value) -> ResolverResult>>,
}

impl<TCtx> Router<TCtx> {
    pub fn new() -> Self {
        Self {
            query: BTreeMap::new(),
            mutation: BTreeMap::new(),
        }
    }

    pub fn query<TType, TArgs: DeserializeOwned + 'static, TResolver: Resolver<TType> + 'static>(
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
        self
    }

    pub fn mutation<
        TType,
        TArgs: DeserializeOwned + 'static,
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
        self
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

    #[tokio::test]
    async fn deserialize_test() {
        let router = Router::<()>::new()
            .query("null", |_, _: ()| ())
            .query("string", |_, _: ()| "Hello World")
            .query("number", |_, _: ()| 42i32)
            .query("bool", |_, _: ()| true);

        let result = router.exec_query((), "null", json!(null)).await.unwrap();
        assert_eq!(result, json!(null));

        let result = router.exec_query((), "string", json!(null)).await.unwrap();
        assert_eq!(result, json!("Hello World"));

        let result = router.exec_query((), "number", json!(null)).await.unwrap();
        assert_eq!(result, json!(42));

        let result = router.exec_query((), "bool", json!(null)).await.unwrap();
        assert_eq!(result, json!(true));
    }

    #[derive(Deserialize, Serialize)]
    pub struct MyArgs {
        pub val: String,
    }

    #[tokio::test]
    async fn with_args_test() {
        let router = Router::<()>::new()
            .query("string", |_, v: String| v)
            .query("number", |_, v: i32| v)
            .query("bool", |_, v: bool| v)
            .query("custom_sruct", |_, v: MyArgs| v.val);

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
                "custom_sruct",
                MyArgs {
                    val: "Testing".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(result, json!("Testing"));
    }
}
