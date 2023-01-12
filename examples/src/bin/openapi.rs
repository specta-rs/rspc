//! This is more of a proof on concept than feature. Be aware will 110% run into problems using it.
//!
// use axum::{extract::Path, routing::get};
// use rspc::{
//     plugins::openapi::{Method, OpenAPI, OpenAPIConfig},
//     Type,
// };
// use serde::{Deserialize, Serialize};
// use tower_http::cors::{Any, CorsLayer};

// #[derive(Serialize, Deserialize, Type)]
// pub struct User {
//     pub name: String,
//     pub age: u32,
// }

// #[tokio::main]
// async fn main() {
//     let router = <rspc::Router>::new()
//         .query("version", |t| {
//             t(|_, _: ()| async move { env!("CARGO_PKG_VERSION") }).openapi(Method::GET, "/version")
//         })
//         .query("demo", |t| {
//             t(|_, _: i32| async move {
//                 todo!();
//             })
//             .openapi(Method::POST, "/demo")
//         })
//         .query("me", |t| {
//             t(|_, _: ()| async move {
//                 User {
//                     name: "Oscar Beaumont".into(),
//                     age: 20,
//                 }
//             })
//             .openapi(Method::GET, "/me")
//         })
//         .query("users", |t| {
//             t(|_, _: ()| async move {
//                 vec![User {
//                     name: "Monty Beaumont".into(),
//                     age: 8,
//                 }]
//             })
//             .openapi(Method::GET, "/users")
//         })
//         .mutation("createUser", |t| {
//             t(|_, _: User| async move {
//                 todo!();
//             })
//             .openapi(Method::POST, "/users")
//         })
//         .build()
//         .arced(); // This function is a shortcut to wrap the router in an `Arc`.

//     let app = axum::Router::new()
//         .route("/", get(|| async { "Hello 'rspc'!" }))
//         .route(
//             "/api/*path",
//             router
//                 .clone()
//                 .openapi_endpoint(
//                     OpenAPIConfig {
//                         title: "Demo API".into(),
//                         version: "1.0.0".into(),
//                         base_url: "http://[::]:4000/api".into(),
//                     },
//                     |path: Path<String>| {
//                         println!("Client requested operation '{}'", *path);
//                     },
//                 )
//                 .axum(),
//         )
//         // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
//         .route(
//             "/rspc/:id",
//             router
//                 .endpoint(|path: Path<String>| {
//                     println!("Client requested operation '{}'", *path);
//                 })
//                 .axum(),
//         )
//         // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
//         .layer(
//             CorsLayer::new()
//                 .allow_methods(Any)
//                 .allow_headers(Any)
//                 .allow_origin(Any),
//         );

//     let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
//     println!("listening on http://{}/rspc/version", addr);
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }

fn main() {
    // OpenAPI is not ready for prime time so it has been commented out for now.
    // I will be working to bring this back in the near future.

    todo!();
}
