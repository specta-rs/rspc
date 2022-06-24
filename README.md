**This project is a work in progress. I just wanted to try and replicate the syntax of [trpc](https://trpc.io/) in Rust**

# trpc-rs

A blazing fast and easy to use TRPC server for Rust.

## Example

You define a `trpc` router and attach resolvers to it like below. This will be very familiar if you have used [trpc](https://trpc.io/) or [GraphQL](https://graphql.org) before.

```rust
let router = trpc_rs::Router::<()>::new()
    .query("version", |_| "0.0.1")
    .mutation("helloWorld", |_| async { "Hello World!" });
```

## Features:

 - Per Request Context - Great for database connection & authentication data

## Planned Features:

 - Pass argument type earlier in query/mutation.
 - Cleanup the codebase -> Currently their is a lot of duplicate code between `mutations` and `queries`. Add comments to everything!
 - Axum example
 - Msgpack support
 - Support for multiple queries in single request
 - [Merging servers](https://trpc.io/docs/merging-routers)
 - [Middleware](https://trpc.io/docs/middlewares) & [Route Meta](https://trpc.io/docs/metadata)
 - [Error Handling](https://trpc.io/docs/error-handling)
 - [Subscriptions](https://trpc.io/docs/subscriptions)
 - Input validation
 - Exporting input validation to frontend code so it can be reused
 - Fix `ts-rs` [#70](https://github.com/Aleph-Alpha/ts-rs/issues/70)
 - Documentation