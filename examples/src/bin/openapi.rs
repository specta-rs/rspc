use axum::{extract::Path, routing::get};
use rspc::plugins::openapi::{Method, OpenAPI, OpenAPIConfig};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = <rspc::Router>::new()
        .query("version", |t| {
            t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") }).openapi(Method::GET, "/version")
        })
        .query("users", |t| {
            t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") }).openapi(Method::POST, "/users")
        })
        .build()
        .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .route(
            "/api/*path",
            router
                .clone()
                .openapi_endpoint(
                    OpenAPIConfig {
                        title: "Demo API",
                        version: "1.0.0",
                        base_url: "http://[::]:4000/api",
                    },
                    |path: Path<String>| {
                        println!("Client requested operation '{}'", *path);
                        ()
                    },
                )
                .axum(),
        )
        .route(
            "/rspc/:id",
            router
                .endpoint(|path: Path<String>| {
                    println!("Client requested operation '{}'", *path);
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
