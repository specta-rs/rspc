<h1 align="center">trpc-rs</h1>
<p align="center">🚧 Work in progress 🚧</p>
<div align="center">
 <strong>
   A blazingly fast and easy to use TRPC server for Rust.
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/trpc-rs">
    <img src="https://img.shields.io/crates/v/trpc-rs.svg?style=flat-square"
    alt="crates.io" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/trpc-rs">
    <img src="https://img.shields.io/crates/d/trpc-rs.svg?style=flat-square"
      alt="download count badge" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/trpc-rs">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs" />
  </a>
</div>
<br/>

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

 - [Batch queries](https://trpc.io/docs/links)
 - [Merging servers](https://trpc.io/docs/merging-routers)
 - [Middleware](https://trpc.io/docs/middlewares) & [Route Meta](https://trpc.io/docs/metadata)
 - [Error Handling](https://trpc.io/docs/error-handling)
 - Msgpack support
 
 - Unit test for exporting types and validating schema
 - Pass argument type earlier in query/mutation -> Expose this as helper which user can call
 - [Subscriptions](https://trpc.io/docs/subscriptions)
 - Cleanup the codebase -> Currently their is a lot of duplicate code between `mutations` and `queries`. Add comments to everything!

 - Warn user when not all variants of enum were registered -> Make it so they can turn this into unit test
 - Passthrough Rust [deprecated](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-deprecated-attribute) to [Typescript doc comment](https://stackoverflow.com/questions/60755711/is-it-possible-to-mark-something-as-deprecated-in-typescript)
 - Input validation
 - Exporting input validation to frontend code so it can be reused
 - Fix `ts-rs` [#70](https://github.com/Aleph-Alpha/ts-rs/issues/70)
 - TRPC Rust Client
 - Documentation