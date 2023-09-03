//! Using tower_cookies as an Axum extractor right now is the best way to work with cookies from rspc.
//! An official API will likely exist in the future but this works well for now.
use std::{ops::Add, path::PathBuf};

use axum::routing::get;
use rspc::{integrations::httpz::Request, ExportConfig, Rspc};
use time::OffsetDateTime;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct Ctx {
    cookies: Cookies,
}

const R: Rspc<Ctx> = Rspc::new();

#[tokio::main]
async fn main() {
    let router = R
        .router()
        .procedure(
            "getCookie",
            R.query(|ctx, _: ()| {
                Ok(ctx
                    .cookies
                    .get("myDemoCookie")
                    .map(|c| c.value().to_string()))
            }),
        )
        .procedure(
            "setCookie",
            R.mutation(|ctx, new_value: String| {
                let mut cookie = Cookie::new("myDemoCookie", new_value);
                cookie.set_expires(Some(OffsetDateTime::now_utc().add(time::Duration::DAY)));
                cookie.set_path("/"); // Ensure you have this or it will default to `/rspc` which will cause issues.
                ctx.cookies.add(cookie);

                Ok(())
            }),
        )
        .build()
        .unwrap()
        .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
        ))
        .unwrap();

    let app = axum::Router::new()
        .with_state(())
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest(
            "/rspc",
            router
                .endpoint(|mut req: Request| {
                    // TODO: This API is going to be replaced with a httpz cookie manager in the next release to deal with Axum's recent changes.
                    let cookies = req
                        .deprecated_extract::<Cookies, ()>()
                        .expect("The Axum state doesn't match the router. Ensure you added `with_state(T)` where `T` matches the second generic!")
                        .unwrap();
                    Ctx { cookies }
                })
                .axum(),
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
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
