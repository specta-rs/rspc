use std::{path::PathBuf, sync::Arc};

use example::{basic, selection, subscriptions};

use axum::routing::get;
use rspc::{integrations::httpz::Request, Config, Router};
use tower_http::cors::{Any, CorsLayer};

fn mount() -> Arc<Router<()>> {
    let r1 = Router::<i32>::new().query("demo", |t| t(|_, _: ()| "Merging Routers!"));

    <rspc::Router>::new()
        .config(
            Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ),
        )
        // Basic query
        .query("version", |t| {
            t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") })
        })
        .merge("basic.", basic::mount())
        .merge("subscriptions.", subscriptions::mount())
        .merge("selection.", selection::mount())
        // This middleware changes the TCtx (context type) from `()` to `i32`. All routers being merge under need to take `i32` as their context type.
        .middleware(|mw| mw.middleware(|ctx| async move { Ok(ctx.with_ctx(42i32)) }))
        .merge("r1.", r1)
        .build()
        .arced() // This function is a shortcut to wrap the router in an `Arc`.
}

#[tokio::main]
async fn main() {
    let router = mount();

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest(
            "/rspc",
            router
                .endpoint(|req: Request| {
                    println!("Client requested operation '{}'", req.uri().path());
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_rspc_router() {
        super::mount();
    }
}
