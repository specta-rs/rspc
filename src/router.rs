use std::collections::BTreeMap;

use serde_json::Value;

use crate::{Context, Request, RequestKind, Resolver, ResolverResult};

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

    pub async fn exec(&self, ctx: TCtx, req: Request) -> Result<Value, ()> {
        let name: &str = &req.name;
        let result = match req.kind {
            RequestKind::Query => self.query.get(name).ok_or(())?(Context { ctx, args: () }),
            RequestKind::Mutation => self.mutation.get(name).ok_or(())?(Context { ctx, args: () }),
        };

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

        let req = Request {
            name: "null".into(),
            kind: RequestKind::Query,
        };
        let result = router.exec((), req).await.unwrap();
        assert_eq!(result, serde_json::Value::Null);

        let req = Request {
            name: "string".into(),
            kind: RequestKind::Query,
        };
        let result = router.exec((), req).await.unwrap();
        assert_eq!(result, serde_json::Value::String("Hello World".into()));

        let req = Request {
            name: "number".into(),
            kind: RequestKind::Query,
        };
        let result = router.exec((), req).await.unwrap();
        assert_eq!(result, serde_json::Value::Number(42i32.into()));

        let req = Request {
            name: "bool".into(),
            kind: RequestKind::Query,
        };
        let result = router.exec((), req).await.unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }
}
