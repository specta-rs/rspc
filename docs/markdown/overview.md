---
title: Overview
header: rspc
index: 1
---

**A blazing fast and easy to use TRPC-like server for Rust.**

```rust
let router = <Router>::new()
    .query("version", |ctx, args: ()| "1.0.0")
    .mutation("createUser", |ctx, user_id: i32| User::create(user_id))
    .build();
```
[View more examples](https://github.com/oscartbeaumont/rspc/tree/main/examples)

### Project status

🚧 Work in progress 🚧

Expect breaking changes however it should be stable enough to build a project.

### Introduction

**rspc** is a library which helps you built completely typesafe APIs with a Rust backend and Typescript frontend.

Your `rspc::Router` is converted into [Typescript](https://www.typescriptlang.org) types which can be used on the frontend to prevent simple mistakes from ending up in production!

This library fits a use case between REST and [GraphQL](https://graphql.org). It allows you to built API's with the typesafey of GraphQL without the complexity involved.

rspc acts as a type safe router on top of whatever HTTP or Websocket server you are already using such as [Axum](https://github.com/tokio-rs/axum).

### Features

- ✅ **End-to-end typesafety** - Call your Rust code from Typescript with complete typesafety!
- ✅ **Per Request Context** - Great for database connection & authentication data
- ✅ **Middleware** - For extending your resolvers with auth, logging and more
- ✅ **Merging routers** - Great for separating code between files
- ✅ **Minimal runtime** - Near zero runtime footprint

### Inspiration

This project is based off [trpc](https://trpc.io) and was inspired by the bridge system [Jamie Pine](https://github.com/jamiepine) designed for [Spacedrive](https://www.spacedrive.com). A huge thanks to everyone who helped inspire this project!

### Roadmap

Refer to [GitHub Issue](https://github.com/oscartbeaumont/rspc/issues/2).