use std::collections::BTreeMap;

use serde_json::Value;

use crate::{Context, Resolver, ResolverResult};

pub struct Router<TCtx: 'static> {
    query: BTreeMap<&'static str, Box<dyn Fn(Context<TCtx>) -> ResolverResult>>,
    mutation: BTreeMap<&'static str, Box<dyn Fn(Context<TCtx>) -> ResolverResult>>,
}

impl<TCtx> Router<TCtx> {
    pub fn new() -> Self {
        Self {
            query: BTreeMap::new(),
            mutation: BTreeMap::new(),
        }
    }

    pub fn query<TType, TResolver: Resolver<TType> + 'static>(
        mut self,
        name: &'static str,
        resolver: fn(Context<TCtx>) -> TResolver,
    ) -> Self {
        if self.query.contains_key(name) {
            panic!("trpc-rs error: query with name '{}' already exists", name);
        }

        self.query
            .insert(name, Box::new(move |ctx| resolver(ctx).resolve()));
        self
    }

    pub fn mutation<TType, TResolver: Resolver<TType> + 'static>(
        mut self,
        name: &'static str,
        resolver: fn(Context<TCtx>) -> TResolver,
    ) -> Self {
        if self.mutation.contains_key(name) {
            panic!("trpc-rs error: query with name '{}' already exists", name);
        }

        self.mutation
            .insert(name, Box::new(move |ctx| resolver(ctx).resolve()));
        self
    }

    pub async fn exec_query<S: AsRef<str>>(&self, ctx: TCtx, name: S) -> Result<Value, ()> {
        let name = name.as_ref();
        let result = self.query.get(name).ok_or(())?(Context { ctx, args: () });

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

    pub async fn exec_mutation<S: AsRef<str>>(&self, ctx: TCtx, name: S) -> Result<Value, ()> {
        let name = name.as_ref();
        let result = self.mutation.get(name).ok_or(())?(Context { ctx, args: () });

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
    use super::*;

    #[tokio::test]
    async fn basic_test() {
        let router = Router::<()>::new()
            .query("null", |_| ())
            .query("string", |_| "Hello World")
            .query("number", |_| 42i32)
            .query("bool", |_| true);

        let result = router.exec_query((), "null").await.unwrap();
        assert_eq!(result, serde_json::Value::Null);

        let result = router.exec_query((), "string").await.unwrap();
        assert_eq!(result, serde_json::Value::String("Hello World".into()));

        let result = router.exec_query((), "number").await.unwrap();
        assert_eq!(result, serde_json::Value::Number(42i32.into()));

        let result = router.exec_query((), "bool").await.unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }
}
