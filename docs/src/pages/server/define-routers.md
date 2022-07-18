---
title: Define a Router
layout: ../../layouts/MainLayout.astro
---

A router contains a collection of operations (queries, mutations or subscriptions) that can be called by a client. A router has many generic arguments which can be configured by the user to match the type of data that the router will be handling.

A router is defined as `Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>`.

These generic arguments are:
- `TCtx` - The context type that the router will accept when called. This would contain state coming from the webserver such as a user session or database connection.
- `TMeta` - The type of the metadata which can be attached to routes to configure the behaviour of the middleware.
- `TQueryKey` - The type to use for the query key. This is used to identify the query that is being called.
- `TMutationKey` - The type to use for the mutation key. This is used to identify the mutation that is being called.
- `TSubscriptionKey` - The type to use for the subscription key. This is used to identify the subscription that is being called.

```rust
// Create a router with the default configuration.
let router = <Router>::new();

// A router with a custom context
struct MyCtx {}
let router = Router<MyCtx>::new();

// A router which uses integer keys
let router = Router::<(), (), i32, i32, i32>::new();
```

### Add Query Operation

Use the `.query` method to attach a query operation to the router.

```rust
let router = <Router>::new()
    .query("version", |ctx, arg: ()| "1.0.0")
    .build(); // Ensure you build once you have added all your operations.
```

### Add Mutation Operation

Use the `.mutation` method to attach a mutation operation to the router.

```rust
let router = <Router>::new()
    .mutation("createUser", |ctx, arg: ()| todo!())
    .build(); // Ensure you build once you have added all your operations.
```

### Add Subscription Operation

Use the `.subscription` method to attach a subscription operation to the router.

```rust
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

### Method chaining

When combining multiple operations, you must ensure you chain the method calls or shadow the variable. This is required due to the way the generics work on the Router.

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
