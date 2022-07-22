use rspc::{ActualMiddlewareResult, Config, MiddlewareResult, Router};
use serde::Deserialize;
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, TS, Deserialize)]
pub struct LibraryArgs {
    pub library_id: String,
}

#[derive(Debug, TS, Deserialize)]
pub struct DemoArgs {
    pub demo: String,
}

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .config(Config::new().export_ts_bindings("./examples/ts"))
        .query("version", |_ctx, _: ()| env!("CARGO_PKG_VERSION"))
        .middleware(|ctx, args: LibraryArgs, next| async move {
            println!("MIDDLEWARE: {:?}", args);
            match next(ctx)? {
                MiddlewareResult::Stream(stream) => Ok(stream.into_middleware_result()),
                result => Ok(result.await?.into_middleware_result()),
            }
        })
        .query("libraryThings", |ctx, _: ()| {
            // TODO: Get the args into here
            "Another Handler"
        })
        // TODO: Handle having scalar args on this query -> Eg. String
        .query("libraryThingsWithArg", |ctx, arg: DemoArgs| {
            // TODO: Get the args into here
            "Another Handler"
        })
        .build();
}
