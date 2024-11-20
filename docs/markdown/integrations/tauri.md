---
title: Tauri
index: 41
---

rspc has a built-in integration with [Tauri](https://tauri.app/) so that you can expose your API to your frontend code using Tauri's IPC.

### Enable feature

For the integration to work you must enable the `tauri` feature of rspc. Ensure the rspc line in your `Cargo.toml` file looks like the following:

```toml
[dependencies]
rspc = { version = "0.0.0", features = ["tauri"] } # Ensure you put the latest version!
```

Read more about Rust features [here](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features)

### Usage

Then expose your router using the Tauri plugin.

```rust
let router = <Router>::new().build();

tauri::Builder::default()
    .plugin(rspc::integrations::tauri::plugin(router, || ()))
```

### Usage on frontend

```typescript
import { createClient } from '@rspc/client';
import { TauriTransport } from '@rspc/tauri';
import type { Procedures } from "./ts/bindings"; // These were the bindings exported from your Rust code!

const client = createClient<Procedures>({
	transport: new TauriTransport()
});

client.query(['version']).then((data) => console.log(data));
```

You can use the `client` by itself or integrate with the [Tanstack Query](/client/tanstack-query) hooks.
