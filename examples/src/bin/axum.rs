use std::path::PathBuf;

use example::{basic, selection, subscriptions};

use axum::{extract::Path, routing::get};
use rspc::{integrations::httpz::Request, Config, Router};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let r1 = Router::<i32>::new().query("demo", |t| t(|_, _: ()| "Merging Routers!"));

    let router =
        <rspc::Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            // Basic query
            .query("version", |t| {
                t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") })
            })
            .merge("basic.", basic::mount())
            .merge("subscriptions.", subscriptions::mount())
            .merge("selection.", selection::mount())
            // This middleware changes the TCtx (context type) from `()` to `i32`. All routers being merge under need to take `i32` as their context type.
            .middleware(|mw| mw.middleware(|ctx| async move { return Ok(ctx.with_ctx(42i32)) }))
            .merge("r1.", r1)
            .build()
            .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    let app = axum::Router::new()
        .with_state(())
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest(
            "/rspc",
            router
                .endpoint(|mut req: Request| {
                    // Official rspc API
                    println!("Client requested operation '{}'", req.uri().path());

                    // Deprecated Axum extractors - this API will be removed in the future
                    // The first generic is the Axum extractor and the second is the type of your Axum state.
                    // If the state generic is wrong you will get a **RUNTIME** error so be careful!
                    // TODO: Be aware these will NOT work for websockets. If this is a problem for you open an issue on GitHub!
                    let path = req
                        .deprecated_extract::<Path<String>, ()>()
                        .expect("I got the Axum state type wrong!")
                        .unwrap();
                    println!("Client requested operation '{}'", path.0);

                    ()
                })
                .axum(),
        )
        // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
