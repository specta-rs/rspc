//! Using tower_cookies as an Axum extractor right now is the best way to work with cookies from rspc.
//! An official API will likely exist in the future but this works well for now.
use std::{ops::Add, path::PathBuf};

use axum::routing::get;
use rspc::Config;
use time::OffsetDateTime;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::cors::{Any, CorsLayer};

pub struct Ctx {
    cookies: Cookies,
}

#[tokio::main]
async fn main() {
    let router =
        rspc::Router::<Ctx>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
            ))
            .query("getCookie", |t| {
                t(|ctx, _: ()| {
                    ctx.cookies
                        .get("myDemoCookie")
                        .map(|c| c.value().to_string())
                })
            })
            .mutation("setCookie", |t| {
                t(|ctx, new_value: String| {
                    let mut cookie = Cookie::new("myDemoCookie", new_value);
                    cookie.set_expires(Some(OffsetDateTime::now_utc().add(time::Duration::DAY)));
                    cookie.set_path("/"); // Ensure you have this or it will default to `/rspc` which will cause issues.
                    ctx.cookies.add(cookie);
                })
            })
            .build()
            .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    let app = axum::Router::new()
        .with_state(())
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest(
            "/rspc",
            rspc_axum::endpoint(router, |cookies: Cookies| Ctx { cookies }),
        )
        .layer(CookieManagerLayer::new())
        // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
