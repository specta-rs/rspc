---
title: Request Context
---

When calling execute on a operation you must provide a request context. The type of the request context must match the `TCtx` generic parameter defined on the `rspc::Router`.

Using request context is important because it means you can construct the router without a dependency on anything (such a database) which allows you to validate the router in a unit test. The routes are stringly typed so we can't just rely on Rust's compiler to validate the router. This tradeoff was made for the superior developer experience as we believe using request context and a unit test for validating the router is able to mitigate the risk.

A request context is created on every request and can hold any data the user wants. The request context abstracts the underlying transport layer (such as HTTP, Websocket or Tauri) so that the router can be agonistic about the transport layer.

```rust
struct MyCtx {
    some_value: &'static str,
}

let router = Router::<MyCtx>::new()
    .query("myQuery", |t| t(|ctx, input: ()| {
        assert_eq!(ctx.some_value, "Hello World");
    }))
    .build();

// You will usually provide a closure to the rspc integration which returns this.
let request_ctx = MyCtx {
    some_value: "Hello World",
};

// You won't call this directly, it will be done by an integration.
let result: StreamOrValue = router
    .exec(
        request_ctx,
        OperationKind::Query,
        OperationKey("myQuery".into(), None),
    )
    .await
    .unwrap();
```
