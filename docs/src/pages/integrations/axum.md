---
title: Axum Integration
layout: ../../layouts/MainLayout.astro
---

**rspc** has a built-in integration with [Axum](https://github.com/tokio-rs/axum) so that you can expose your API over HTTP.

### Enable feature

For the integration to work you use enable the `axum` feature of **rspc**.

```toml
[dependencies]
rspc = { version = "0.0.0", features = ["axum"] } # Ensure you put the latest version!
```

### Usage

```rust
let router = rspc::Router::<()>::new()
    .query("version", |_, _: ()| "1.0.0");
let router = Arc::new(router.build());

let app = axum::Router::new()
    .route("/", get(|| async { "Hello 'rspc'!" }))
    .route("/trpc/:id", router.axum_handler(|| ()))
    .layer(cors);
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/axum.rs)