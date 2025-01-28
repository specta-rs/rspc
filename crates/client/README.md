# Rust client

[![docs.rs](https://img.shields.io/crates/v/rspc-client)](https://docs.rs/rspc-client)

> [!CAUTION]
> This crate is still a work in progress. You can use it but we can't guarantee that it's API won't change.

Allows you to make queries from a Rust client to an rspc server.

## Example

```rust
// This file is generated via the `rspc::Rust` language on your server
mod bindings;

#[tokio::main]
async fn main() {
    let client = rspc_client::Client::new("http://[::]:4000/rspc");

    println!("{:?}", client.exec::<bindings::version>(()).await);
    println!(
        "{:?}",
        client
            .exec::<bindings::echo>("Some random string!".into())
            .await
    );
    println!(
        "{:?}",
        client
            .exec::<bindings::sendMsg>("Hello from rspc Rust client!".into())
            .await
    );
}
```
