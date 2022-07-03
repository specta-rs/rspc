---
title: Axum Integration
layout: ../../layouts/MainLayout.astro
---

**rspc** has a built-in integration with [Axum](https://github.com/tokio-rs/axum) so that you can expose your API over HTTP.

### Enable feature

For the integration to work you must enable the `axum` feature of **rspc**. Ensure the rspc line in your `Cargo.toml` file looks like the following:

```toml
[dependencies]
rspc = { version = "0.0.0", features = ["axum"] } # Ensure you put the latest version!
```

Read more about Rust features [here](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features)

### Usage

```rust
let router = rspc::Router::<()>::new()
    .query("version", |_, _: ()| "1.0.0");
let router = Arc::new(router.build());

let app = axum::Router::new()
    .route("/", get(|| async { "Hello 'rspc'!" }))
    .route("/rspc/:id", router.clone().axum_handler(|| ()))
    .route("/rspcws", router.axum_ws_handler(|| ()))
    .layer(cors);
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/axum.rs)