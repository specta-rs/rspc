<p align="center">
 <img width="150" height="150" src="/docs/public/logo.png" alt="Logo">
</p>
<h1 align="center">rspc</h1>
<p align="center">🚧 Work in progress 🚧</p>
<div align="center">
 <strong>
   A blazing fast and easy to use TRPC-like server for Rust.
 </strong>
</div>
<a align="center" href="https://rspc.otbeaumont.me">
  <p>Website</p>
</a>

<br />


<div align="center">
  <a href="https://discord.gg/4V9M5sksw8"><img src="https://img.shields.io/discord/1011665225809924136?style=flat-square" alt="Discord"></a>
  <a href="https://crates.io/crates/rspc">
    <img src="https://img.shields.io/crates/v/rspc.svg?style=flat-square"
    alt="crates.io" />
  </a>
  <a href="https://crates.io/crates/rspc">
    <img src="https://img.shields.io/crates/d/rspc.svg?style=flat-square"
      alt="download count badge" />
  </a>
  <a href="https://docs.rs/rspc">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs" />
  </a>
  <a href="https://www.npmjs.com/package/@rspc/client">
    <img alt="npm (scoped)" src="https://img.shields.io/npm/v/@rspc/client?style=flat-square">
  </a>
</div>
<br/>

## Example

You define a `rspc` router and attach procedures to it like below. This will be very familiar if you have used [trpc](https://trpc.io/) or [GraphQL](https://graphql.org) before.

```rs
let router = <rspc::Router>::new()
    .query("version", |t| {
        t(|ctx, input: ()| "0.0.1")
    })
    .mutation("helloWorld", |t| {
        t(|ctx, input: ()| async { "Hello World!" })
    });
```

## Features:

 - Per Request Context - Great for database connection & authentication data
 - Middleware - With support for context switching
 - Merging routers - Great for separating code between files

### Inspiration

This project is based off [trpc](https://trpc.io) and was inspired by the bridge system [Jamie Pine](https://github.com/jamiepine) designed for [Spacedrive](https://www.spacedrive.com). A huge thanks to everyone who helped inspire this project!
