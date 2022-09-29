---
title: Breaking Changes
index: 3
---

# 0.0.5 to 0.0.6 - rspc

This release comes with a huge amount of breaking changes. These changes are going to allow for many benefits in the future such as a rich plugin ecosystem. If your having trouble upgrading open a GitHub Issue or jump in the Discord server. New [rspc vscode extension](https://marketplace.visualstudio.com/items?itemName=oscartbeaumont.rspc-vscode) too!

### Httpz integration

This update moves from offering a direct [Axum](https://github.com/tokio-rs/axum) integration to using [httpz](https://github.com/oscartbeaumont/httpz). This change is going to allow rspc to support other HTTP servers and serverless environments in the near future.

Rust Changes:

```diff
let app = axum::Router::new()
    .route("/", get(|| async { "Hello 'rspc'!" }))
-   .route("/rspc/:id", router.clone().axum_handler(|| ()))
-   .route("/rspcws", router.axum_ws_handler(|| ()))
+   .route("/rspc/:id", router.endpoint(|req, cookies| ()).axum())
```

Typescript Changes:

```diff
const client = createClient<Operations>({
-   transport: new WebsocketTransport("ws://localhost:8080/rspcws"),
+   transport: new WebsocketTransport("ws://localhost:8080/rspc/ws"),
});
```

### New Typescript bindings format

The internal format of the generated Typescript bindings has changed. The import has also changed so ensure you update your code as follows.

```diff
- import type { Operations } from "./my-bindings";
+ import type { Procedures } from "./my-bindings";
```

### New middleware syntax

```diff
let router = Router::new()
- .middleware(|ctx| async move {
-     println!("MIDDLEWARE TWO");
-     ctx.next("hello").await
- })
+ .middleware(|mw| {
+    mw.middleware(|mw| async move {
+        let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
+        // state allows sharing data between the two closures and is optional.
+        Ok(mw.with_state(state).with_ctx("hello"))
+    })
+    // The .resp() part is optional and only required if you need to modify the return value.
+    // Be aware it will be called for every value in a subscription stream.
+    .resp(|state, result| async move {
+        println!(
+            "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
+            state.0, state.1, state.2, result
+        );
+        Ok(result)
+    })
+})
```

### New procedure syntax

The new procedure syntax is one of the biggest changes with this release. It is reccomended you install the [Visual Studio Code extension](https://marketplace.visualstudio.com/items?itemName=oscartbeaumont.rspc-vscode) which will provide many snippets to make working with rspc as easy as possible.

#### Query

```diff
let router = Router::new()
- .query("version", |ctx, input: ()| "1.0.0")
+ .query("version", |t| t(|ctx, input: ()| "1.0.0"))
```

#### Mutation

```diff
let router = Router::new()
- .mutation("demo", |ctx, input: ()| async move { todo!() })
+ .mutation("demo", |t| t(|ctx, input: ()| async move { todo!() })
```

#### Subscription

Rust:

```diff
let router = Router::new()
- .subscription("version", |ctx, input: ()| stream! { yield 42; })
+ .subscription("version", |t| t(|ctx, input: ()| stream! { yield 42; }))
```

Typescript:

```diff
rspc.useSubscription(['my.subscription'], {
-    onNext: (data) => {
+    onData: (data) => {
        console.log(data)
    }
});
```

### Minor changes/new features

 - `ws`, `rpc.*` and `rspc.*` are now reserved names for procedures
 - `@rspc/solid` has been upgraded to the new `@tanstack/solid-query`.
 - `router.execute` now returns a `serde_json::Value` and does not support subscriptions. Use `router.execute_subscription` to execute a subscription.
 - The Axum body type is now `Vec<u8>` not `hyper::Body`. This may cause issues if your extractors aren't generic enough.
 - Support for up to 5 Axum extractors. This limit will increase to 16 in a future release.
 
### Warning on unstable API's

 - Support for Axum extractors will likely change or be removed in the future when support for other HTTP servers is added. It exists in this release for backwards compatibility.

# 0.0.5 to 0.0.6 - @rspc/client

All of the frontend code have been split up into multiple npm packages. We now have [`@rspc/client`](https://www.npmjs.com/package/@rspc/client), [`@rspc/react`](https://www.npmjs.com/package/@rspc/react) and [`@rspc/tauri`](https://www.npmjs.com/package/@rspc/tauri). This will help with SSR and reducing project dependencies.

Start by installing the new packages if your require them.

```bash
npm install @rspc/react # If your using the React hooks
npm install @rspc/tauri # If your using the Tauri transport
```

Then change your imports as follows. From:

```ts
import { createReactQueryHooks } from '@rspc/client';
import { TauriTransport } from '@rspc/client';
```

To:

```ts
import { createReactQueryHooks } from '@rspc/react';
import { TauriTransport } from '@rspc/tauri';
```

There is no Rust release for these changes. I am going to look into following SemVer in a future release, this is just a quick patch as multiple people have reported this being an issue with [Next.js](https://nextjs.org) SSR.
