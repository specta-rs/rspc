---
title: Middleware
---

**rspc** allows adding middleware to your router which can intercept the request and response for operations defined after it on the router. The middleware can modify the request context as well as modify the response.

```rust
use rspc::Router;

fn main() {
    let router = Router::<()>::new()
        .query("version", |t| {
            t(|ctx: (), _: ()| {
                println!("ANOTHER QUERY");
                env!("CARGO_PKG_VERSION")
            })
        })
        .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx(42)) }))
        .query("anotherQuery", |t| t(|ctx: i32, _: ()| "Hello World!"))
        .middleware(|mw| mw.middleware(|mw| async move { Ok(mw.with_ctx("Hello World")) }))
        .query("myExtraCoolQuery", |t| {
            t(|ctx: &'static str, _: ()| "Another Result!")
        })
        .build();
}
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/middleware.rs)

### Context switching

Middleware are allowed to modify the context. This includes being able to change it's type. All operations below the middleware in the router will receive the new context type.

```rust
use rspc::Router;

fn main() {
    let router = Router::<()>::new()
        .middleware(|mw| mw.middleware(|mw| async move {
            let old_ctx: () = mw.ctx;
            Ok(mw.with_ctx(42))
        }))
        .query("version", |t| {
            t(|ctx: i32, _: ()| "1.0.0")
        })
        .query("anotherQuery", |t| t(|ctx: i32, _: ()| "Hello World!"))
        .build();
}
```

### Route metadata

Feature coming soon. Tracking in issue [#21](https://github.com/oscartbeaumont/rspc/issues/21).