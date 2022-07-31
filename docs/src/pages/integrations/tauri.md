---
title: Tauri Integration
layout: ../../layouts/MainLayout.astro
---

**rspc** has a built-in integration with [Tauri](https://tauri.app/) so that you can expose your API to your frontend code using Tauri's IPC.

To expose your router use the Tauri plugin.

```rust
let router = <Router>::new().build();

tauri::Builder::default()
    .plugin(sdcore::rspc::tauri::plugin(router, || ()))
```

### Usage on frontend

```typescript
import { TauriTransport, createClient } from '@rspc/client';
import type { Operations } from "./ts/bindings"; // These were the bindings exported from your Rust code!

const client = createClient<Operations>({
	transport: new TauriTransport()
});

client.query('version').then((data) => console.log(data));
```