use std::net::Ipv6Addr;

use axum::{routing::get, Router};

pub fn mount() -> rspc::Router {
    <rspc::Router>::default() // .procedure("helloWorld", todo!())
}

#[tokio::main]
async fn main() {
    let router = mount();

    rspc_axum::endpoint(router); // TODO: hook this up

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind((Ipv6Addr::UNSPECIFIED, 3000))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// TODO: Unit test for exporting bindings
