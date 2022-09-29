---
title: Convention
---

This page documents conventions for building application with rspc. You should remember that conventions act as a starting place and you should do whatever makes sense for your project or team, even if it goes against the convention.

## Capturing variables

rspc allows for capturing variables in the closure of a resolver. This is generally fround upon as it put a requirement on that value when creating the router which could limit your ability to unit test the router. More of the logic behind this is explained in [request context](/server/request-context). This is a general rule and you will likely find exceptions.

```rust

// NOT-RECOMMEND - Capturing variables
pub(crate) fn mount(db: DatabaseConn) -> Router {
    <Router>::new()
        .query("getUsers", move |t| t(move |_, _: ()| async move { db.users().find_all().exec().await }));
}

// RECOMMEND - Using Request Context
struct MyCtx { db: DatabaseConn }
pub(crate) fn mount() -> Router {
    Router::<MyCtx>::new()
        .query("getUsers", move |t| t(move |ctx: MyCtx, _: ()| async move { ctx.db.users().find_all().exec().await }));
}
```

If the compiler asks you to use the `move` keyword on the resolver closure (not the async block) you are capturing variables.

## Project layout

A conventional folder structure for rspc is as follows.

`src/main.rs`

```rust
mod api;

#[tokio::main]
async fn main() {
    let router = api::mount();

    // Expose your router through whatever integration you like.
}
```

`src/api/mod.rs`

```rust
mod resource1;

pub(crate) type Router = rspc::Router<(), ()>;
pub(crate) type RouterBuilder = rspc::RouterBuilder<(), ()>;

pub(crate) fn mount() -> Router {
    Router::new()
        .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .merge("resource1.", resource1::mount())
        .build()
        .arced();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rspc_router() {
        super::mount();
    }
}
```

`src/api/resource1.rs`

```rust
pub(crate) fn mount() -> RouterBuilder {
    RouterBuilder::new()
        .query("someQuery", |t| t(|_, _: ()| "Hello World"))
}
```
