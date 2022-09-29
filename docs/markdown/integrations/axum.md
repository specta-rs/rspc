---
title: Axum
index: 40
---

rspc has a built-in integration with [Axum](https://github.com/tokio-rs/axum) so that you can expose your API over HTTP.

### Enable feature

For the integration to work you must enable the `axum` feature of rspc. Ensure the rspc line in your `Cargo.toml` file looks like the following:

```toml
[dependencies]
rspc = { version = "0.0.0", features = ["axum"] } # Ensure you put the latest version!
```

Read more about Rust features [here](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features)

### Usage

```rust
let router = rspc::Router::<()>::new()
    .query("version", |_, _: ()| "1.0.0")
    .build()
    .arced();

let app = axum::Router::new()
    .route("/", get(|| async { "Hello 'rspc'!" }))
    .route("/rspc/:id", router.endpoint(|| ()).axum())
    .layer(cors);
```

### Extracting Context from Request

**Warning: The Axum extractor API is probally going to be removed in a future release. If you are using this API, I would appreciate a message in the Discord about your usecase so I can ensure the replacement API can do everything you need.**

**Warning: Current we only support a single extractor. This is a temporary limitation so open a GitHub Issue if you need more.**

You may want to use <a href="https://docs.rs/axum/latest/axum/index.html#extractors" target="_blank">Axum extractors</a> to get data from the request such as cookies and put them on the request context. The `axum_handler` function takes a closure that can take up to 16 valid Axum extractors as arguments and then returns the [request context](/server/request-context) (of type `TCtx`).

```rust
let router = rspc::Router::<String>::new()
    .query("currentPath", |ctx, _: ()| ctx)
    .build()
    .arced();

let app = axum::Router::new()
    .route("/", get(|| async { "Hello 'rspc'!" }))
    // We use Axum `Path` extractor. The `rspc::Router` has `TCtx` set to `String` so we return the path string as the context.
    .route("/rspc/:id", router.endpoint(|Path(path): Path<String>| path).axum())
    .layer(cors);
```

### Usage on frontend

```typescript
import { FetchTransport, WebsocketTransport, createClient } from '@rspc/client';
import type { Procedures } from "./ts/bindings"; // These were the bindings exported from your Rust code!

// For fetch transport
const client = createClient<Procedures>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

// For websocket transport - Required for subscriptions
const client = createClient<Procedures>({
  transport: new WebsocketTransport("ws://localhost:8080/rspc/ws"),
});

client.query(['version']).then((data) => console.log(data));
```