---
title: Quickstart
layout: ../layouts/MainLayout.astro
---

**Get rspc up and running in your own project.**

### (Optional) Create new project

If you haven't got a Rust project already setup, create a new one using the following command.

```bash
cargo new <project-name>
cd <project-name>
cargo add tokio --features full # rpsc requires an async runtime
```

### Install rspc

`rspc` is distributed through a Rust crate hosted on [crates.io](https://crates.io/rspc). Add it to your project using the following command:

```bash
cargo add rspc
```

You may need to install [cargo edit](https://github.com/killercup/cargo-edit) if your not running Rust `1.62.0` or later.

### Create router

Go into `src/main.rs` and add the following code:

```rust
use rspc::Router;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .query("version", |ctx, arg: ()| env!("CARGO_PKG_VERSION"))
        .build();
}
```

Now you have setup a basic `rspc::Router`.