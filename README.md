<h1 align="center">rspc</h1>
<p align="center">🚧 Work in progress 🚧</p>
<div align="center">
 <strong>
   A blazing fast and easy to use TRPC-like server for Rust.
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/rspc">
    <img src="https://img.shields.io/crates/v/rspc.svg?style=flat-square"
    alt="crates.io" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/rspc">
    <img src="https://img.shields.io/crates/d/rspc.svg?style=flat-square"
      alt="download count badge" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/rspc">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs" />
  </a>
</div>
<br/>

## Example

You define a `trpc` router and attach resolvers to it like below. This will be very familiar if you have used [trpc](https://trpc.io/) or [GraphQL](https://graphql.org) before.

```rust
let router = rspc::Router::<()>::new()
    .query("version", |_| "0.0.1")
    .mutation("helloWorld", |_| async { "Hello World!" });
```

## Features:

 - Per Request Context - Great for database connection & authentication data
 - Middleware - With support for context switching
 - Merging routers - Great for separating code between files
