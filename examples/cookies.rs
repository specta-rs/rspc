use std::ops::Add;

use axum::routing::get;
use rspc::Config;
use time::OffsetDateTime;
use tower_cookies::{Cookie, Cookies};
use tower_http::cors::{Any, CorsLayer};

struct Ctx {
    cookies: Cookies,
}

#[tokio::main]
async fn main() {
    let router = rspc::Router::<Ctx>::new()
        .config(Config::new().export_ts_bindings("./packages/example/bindings.ts"))
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

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .route(
            "/rspc/:id",
            router
                .clone()
                .axum_handler(|cookies: Cookies| Ctx { cookies }),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
