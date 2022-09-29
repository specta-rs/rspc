---
title: Middleware
---

**rspc** allows adding middleware to your router which can intercept the request and response for operations defined after it on the router. The middleware can modify the request context as well as modify the response.

```rust
let router = Router<()>::new()
    .middleware(|ctx: ()| async move {
        println!("MIDDLEWARE ONE");
        ctx.next(42).await
    })
    .query("version", |ctx: i32, _: ()| {
        println!("ANOTHER QUERY");
        env!("CARGO_PKG_VERSION")
    })
    .middleware(|ctx| async move {
        println!("MIDDLEWARE TWO");
        ctx.next("hello").await
    })
    .query("another", |ctx: &'static str, _: ()| {
        println!("ANOTHER QUERY");
        "Another Result!"
    })
    .build();
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/middleware.rs)

### Context switching

Middleware are allowed to modify the context. This includes being able to change it's type. All operations below the middleware in the router will receive the new context type.

```rust
let router = <Router>::new()
    // The context defaults to `()`. The `ctx` parameters type will be inferred.
    .middleware(|ctx: MiddlewareContext<(), i32>| async move { ctx.next(42).await })
    // See how the context is now an i32
    .query("version", |ctx: i32, input: ()| "1.0.0")
    .build();
```

### Route metadata

Feature coming soon. Tracking in issue [#21](https://github.com/oscartbeaumont/rspc/issues/21).