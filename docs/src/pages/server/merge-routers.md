---
title: Merging Router
layout: ../../layouts/MainLayout.astro
---

When building an API server, you will often want to split up your endpoints into multiple files to make the code easier to work on. **rspc** allows for merging routes to make doing this easy.

`router.merge(prefix, router)`

```rust
let users_router = <Router>::new()
        .query("list", |ctx, arg: ()| vec![] as Vec<()>);

let router = <Router>::new()
    .query("version", |_ctx, _: ()| "1.0.0")
    .merge("users.", users_router) // The first parameter is a prefix to add to all routes in the merged router.
    .build();
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/merge_routers.rs)

**Merging is currently limited to routers using `&'static str` as their `TQueryKey`, `TMutationKey` and `TSubscription` key. However, this limitation will be lifted in the future.**