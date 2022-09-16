---
title: Router
---

A router contains a collection of operations (queries, mutations or subscriptions) that can be called by a client. A router has many generic arguments which can be configured by the user to match the type of data that the router will be handling.

A router is defined as `Router<TCtx, TMeta>`.

These generic arguments are:

- `TCtx` - The [request context](/server/request-context) type that the router will use. This is usually a `struct` containing state coming from the webserver such as a user session and database connection.
- `TMeta` - The type of the [router metadata](/server/router-metadata) which can be attached to routes to configure the behaviour of the middleware.

```rust
// Create a router with the default configuration.
let router = <Router>::new();

// A router with a custom context struct
struct MyCtx {}
let router = Router<MyCtx>::new();

// A router with a custom context and metdata type
struct MyMeta {}
let router = Router<MyCtx, MyMeta>::new();
```

### Query operation

Use the `.query` method to attach a query operation to the router.

```rust
let router = <Router>::new()
    .query("version", |ctx, arg: ()| "1.0.0")
    .build(); // Ensure you build once you have added all your operations.
```

A query should take a set of arguments and return a result. The resolvers should not have any backend side effects!

### Mutation operation

Use the `.mutation` method to attach a mutation operation to the router.

```rust
let router = <Router>::new()
    .mutation("createUser", |ctx, arg: ()| todo!())
    .build(); // Ensure you build once you have added all your operations.
```

A mutation should take a set of arguments, produce a side effect in the resolver and return a result.

### Subscription operation

Use the `.subscription` method to attach a subscription operation to the router.

```rust
// This example uses the `async_stream` crate.

let router = <Router>::new()
    .subscription("pings", |ctx, arg: ()| async_stream::stream! {
        println!("Client subscribed to 'pings'");
        for i in 0..5 {
            yield "ping".to_string();
            sleep(Duration::from_secs(1)).await;
        }
    })
    .build(); // Ensure you build once you have added all your operations.
```

A subscription should take a set of arguments and then send a stream of results to the client. A subscription goes from the server to the client and is **not** bidirectional!

### Merging routers

When building an API server, you will often want to split up your endpoints into multiple files to make the code easier to work on. **rspc** allows for merging routes to make doing that easy.

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

### Invalidate query

When doing a mutation it is common that you will modify data that is cached on the frontend. This feature is a WIP refer to [issue #19](https://github.com/oscartbeaumont/rspc/issues/19).

### Method chaining

When combining multiple operations, you must ensure you chain the method calls or shadow the router variable. This is required due to the way the generics work on the Router.

```rust
// Chaining method calls
let router = <Router>::new()
    .query("version", |ctx, arg: ()| "1.0.0")
    .mutation("createUser", |ctx, arg: ()| todo!())
    .build();

// Shadowing variable
let router = <Router>::new()
    .query("version", |ctx, arg: ()| "1.0.0")
    .mutation("createUser", |ctx, arg: ()| todo!());
let router = router
    .mutation("deleteUser", |ctx, arg: ()| todo!());
let router = router.build();
```

### Exporting the Typescript bindings

We recommend using the [`export_ts_bindings`](#export_ts_bindings) configuration option but if you need more control over the export process you can call `export_ts_bindings` directly.

```rust
// Be aware the path is relative to where the binary was started.
router.export_ts("./bindings.ts").unwrap();
```

### Router configuration

It is possible to provide configuration to your router by using the `.configure` method. Be aware `.configure` can only be called once as it will replace the existing configuration.

#### export_ts_bindings

will automatically export your Typescript bindings when you build the router as long as Rust debug_assertions are enabled.


```rust
let router = <Router>::new()
    .config(Config::new().export_ts_bindings(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts")))
    .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
    .build();
```

#### set_ts_bindings_header

will add a string you specify to the top of the generated Typescript bindings. This is useful to disable [ESLint](https://eslint.org), [Prettier](https://prettier.io) or other similar tools from processing the generated file.

```rust
let router = <Router>::new()
    .config(Config::new().set_ts_bindings_header("/* eslint-disable */")))
    .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
    .build();
```