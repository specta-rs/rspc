use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use rspc::{ExportConfig, Rspc};

#[derive(Clone)]
pub struct MyCtx {
    count: Arc<AtomicU16>,
}

const R: Rspc<MyCtx> = Rspc::new();

#[tokio::main]
async fn main() {
    let router = R
        .router()
        .procedure(
            "hit",
            // This is a query so it can be accessed in browser without frontend. A `mutation`
            // shoudl be used if the method returns a side effect.
            R.query(|ctx, _: ()| Ok(ctx.count.fetch_add(1, Ordering::SeqCst))),
        )
        .build()
        .unwrap()
        .arced();

    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
        ))
        .unwrap();

    // AtomicU16 provided interior mutability but if your type does not wrap it in an
    // `Arc<Mutex<T>>`. This could be your database connecton or any other value.
    let count = Arc::new(AtomicU16::new(0));

    let app = axum::Router::new().nest(
        "/rspc",
        rspc_httpz::endpoint(router, move || MyCtx { count }).axum(),
    );

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/hit", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
