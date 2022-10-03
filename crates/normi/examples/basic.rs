use axum::routing::get;
use normi::{typed, Object};
use rspc::{Router, Type};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

// #[derive(Serialize, Type, Object)]
// #[normi(name = "org")]
// pub struct Organisation {
//     pub id: String,
//     pub name: String,
//     #[normi(refr)]
//     pub users: Vec<User>,
//     #[normi(refr)]
//     pub owner: User,
//     pub non_normalised_data: Vec<()>,
// }

#[derive(Serialize, Type, Object)]
pub struct User {
    // #[normi(id)]
    pub id: String,
    pub name: String,
}

// #[derive(Serialize, Type, Object)]
// pub struct CompositeId {
//     #[normi(id)]
//     pub org_id: String,
//     #[normi(id)]
//     pub user_id: String,
// }

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .query("version", |t| t(|_, _: ()| "0.1.0"))
        .query("userSync", |t| {
            t(|_, _: ()| User {
                id: "1".to_string(),
                name: "Monty Beaumont".to_string(),
            })
            .map(typed)
        })
        .query("user", |t| {
            t(|_, _: ()| async move {
                Ok(User {
                    id: "1".to_string(),
                    name: "Monty Beaumont".to_string(),
                })
            })
            .map(typed)
        })
        .build()
        .arced();

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .route("/rspc/:id", router.endpoint(|| ()).axum())
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
