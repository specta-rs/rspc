use std::path::PathBuf;

use rspc::{Config, Router};

#[tokio::main]
async fn main() {
    let _r = <Router>::new()
        .config(
            Config::new()
                .export_ts_bindings(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./ts")),
        )
        .middleware(|ctx| async move {
            println!("MIDDLEWARE ONE");
            ctx.next(42).await
        })
        .query("version", |_ctx| {
            println!("ANOTHER QUERY");
            env!("CARGO_PKG_VERSION")
        })
        .middleware(|ctx| async move {
            println!("MIDDLEWARE TWO");
            ctx.next("hello").await
        })
        .query("another", |_ctx| {
            println!("ANOTHER QUERY");
            "Another Result!"
        })
        .build();
}
