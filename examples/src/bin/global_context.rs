use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use rspc::{Config, Router};

#[derive(Clone)]
pub struct MyCtx {
    count: Arc<AtomicU16>,
}

#[tokio::main]
async fn main() {
    let router =
        Router::<MyCtx>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            // This is a query so it can be accessed in browser without frontend. A `mutation`
            // shoudl be used if the method returns a side effect.
            .query("hit", |t| {
                t(|ctx, _: ()| ctx.count.fetch_add(1, Ordering::SeqCst))
            })
            .build()
            .arced();

    // AtomicU16 provided interior mutability but if your type does not wrap it in an
    // `Arc<Mutex<T>>`. This could be your database connecton or any other value.
    let count = Arc::new(AtomicU16::new(0));

    let app = axum::Router::new().nest("/rspc", router.endpoint(move || MyCtx { count }).axum());

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/hit", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
