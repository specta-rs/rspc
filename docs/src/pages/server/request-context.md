---
title: Request Context
layout: ../../layouts/MainLayout.astro
---

When calling execute on a operation you must provide a request context. The type of the request context must match the `TCtx` generic parameter defined on the `rspc::Router`.

```rust
let request_ctx = ();
let result = router.exec_query(request_ctx, "version", json!(null)).await.unwrap();
```