use std::path::PathBuf;

use axum::routing::get;
use normi::{typed, Object};
use rspc::{Config, Router, Type};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, Type, Object)]
#[normi(rename = "org")]
pub struct Organisation {
    #[normi(id)]
    pub id: String,
    pub name: String,
    #[normi(refr)]
    pub users: Vec<User>,
    #[normi(refr)]
    pub owner: User,
    pub non_normalised_data: Vec<()>,
}

#[derive(Serialize, Type, Object)]
pub struct User {
    #[normi(id)]
    pub id: String,
    pub name: String,
}

// TODO: Unit test this duplicate naming

#[derive(Serialize, Type, Object)]
pub struct CompositeId {
    #[normi(id)]
    pub org_id: String,
    #[normi(id)]
    pub user_id: String,
}

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .config(Config::new().export_ts_bindings(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/bindings.ts"),
        ))
        .query("version", |t| t(|_, _: ()| "0.1.0"))
        .query("userSync", |t| {
            t.resolver(|_, _: ()| User {
                id: "1".to_string(),
                name: "Monty Beaumont".to_string(),
            })
            .map(typed)
        })
        .query("user", |t| {
            t.resolver(|_, _: ()| async move {
                Ok(User {
                    id: "1".to_string(),
                    name: "Monty Beaumont".to_string(),
                })
            })
            .map(typed)
        })
        .query("org", |t| {
            t.resolver(|_, _: ()| async move {
                Ok(Organisation {
                    id: "org-1".into(),
                    name: "Org 1".into(),
                    users: vec![
                        User {
                            id: "user-1".into(),
                            name: "Monty Beaumont".into(),
                        },
                        User {
                            id: "user-2".into(),
                            name: "Millie Beaumont".into(),
                        },
                        User {
                            id: "user-3".into(),
                            name: "Oscar Beaumont".into(),
                        },
                    ],
                    owner: User {
                        id: "user-1".into(),
                        name: "Monty Beaumont".into(),
                    },
                    non_normalised_data: vec![(), ()],
                })
            })
            .map(typed)
        })
        .query("composite", |t| {
            t.resolver(|_, _: ()| async move {
                Ok(CompositeId {
                    org_id: "org-1".into(),
                    user_id: "user-1".into(),
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
