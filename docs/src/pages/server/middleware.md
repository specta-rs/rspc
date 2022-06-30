---
title: Middleware
layout: ../../layouts/MainLayout.astro
---

**rspc** allows adding middleware to your router which can intercept the request and response for operations defined after it on the router. The middleware can modify the request context as well as modify the response.

```rust
let router = <Router>::new()
    .middleware(|ctx, next| async move {
        println!("BEFORE");
        let v: Result<serde_json::Value, rspc::ExecError> = next(ctx)?.await;
        println!("AFTER");
        v
    })
    .query("version", |ctx, arg: ()| {
        println!("VERSION");
        "1.0.0"
    })
    .build();
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/middleware.rs)

### Context Switching

As middleware are allow to modify the context, you can replace the context with a new type for all downstream operations.

```rust
let router = <Router>::new()
    // The context defaults to `()`
    .middleware(|ctx: rpsc::Context<()>, next| async move { next(42)?.await })
    // See how the context is now an i32
    .query("version", |ctx: rpsc::Context<i32>, arg: ()| "1.0.0")
    .build();
```