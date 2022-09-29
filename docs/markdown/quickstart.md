---
title: Quickstart
index: 2
---

## Setup your editor

If you are using [Visual Studio Code](https://code.visualstudio.com) you should install the [rspc extension](https://marketplace.visualstudio.com/items?itemName=oscartbeaumont.rspc-vscode) for useful code shortcuts.

## Manual setup

Get rspc up and running in your own project.

### Create new project (optional)

If you haven't got a Rust project already setup, create a new one using the following command.

```bash
cargo new <project-name>
cd <project-name>
cargo add tokio --features full # rpsc requires an async runtime
```

### Install rspc

`rspc` is distributed through a Rust crate hosted on [crates.io](https://crates.io/crates/rspc). Add it to your project using the following command:

```bash
cargo add rspc
```

This command will not exist if your running a Rust version earlier than `1.62.0`, please upgrade your Rust version if this is the case.

### Create router

Go into `src/main.rs` and add the following code:

```rust
use rspc::Router;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .query("version", |t| t(|ctx, input: ()| env!("CARGO_PKG_VERSION")))
        .build();
}
```

Now you have setup a basic `rspc::Router`.

### Exposing your router

Now that you have a router your probably wondering how you access it from your frontend. This is done through an rspc integration. I would recommend starting with [Axum](https://github.com/tokio-rs/axum), by following [this](/integrations/axum).

### Unit test (optional)

Your rspc router is validated on the startup of your application and may panic if anything is incorrect. To ensure you catch any issues with your router before releasing a production version of your application you can use a unit test.

```rust
use rspc::Router;

fn router() -> Router {
    <Router>::new()
        .query("version", |t| t(|ctx, input: ()| env!("CARGO_PKG_VERSION")))
        .build()
}

#[tokio::main]
async fn main() {
    let r = router();
    // Use your router like you normally would
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rspc_router() {
        super::router();
    }
}
```

This method is possible as all context (such as database connections) comes from the request (read more about [request context](/server/request-context)) and therefore you don't need to setup your database and other dependencies to validate the router is valid.
