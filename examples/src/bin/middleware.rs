use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::routing::get;
use rspc::{Config, ErrorCode, MiddlewareContext, Router};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone)]
pub struct UnauthenticatedContext {
    pub session_id: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct User {
    name: String,
}

async fn db_get_user_from_session(_session_id: &str) -> User {
    User {
        name: "Monty Beaumont".to_string(),
    }
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct AuthenticatedCtx {
    user: User,
}

#[tokio::main]
async fn main() {
    let router =
        Router::<UnauthenticatedContext>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            // Logger middleware
            .middleware(|mw| {
                mw.middleware(|mw| async move {
                    let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
                    Ok(mw.with_state(state))
                })
                .resp(|state, result| async move {
                    println!(
                        "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
                        state.0, state.1, state.2, result
                    );
                    Ok(result)
                })
            })
            .query("version", |t| {
                t(|_ctx, _: ()| {
                    println!("ANOTHER QUERY");
                    env!("CARGO_PKG_VERSION")
                })
            })
            // Auth middleware
            .middleware(|mw| {
                mw.middleware(|mw| async move {
                    match mw.ctx.session_id {
                        Some(ref session_id) => {
                            let user = db_get_user_from_session(session_id).await;
                            Ok(mw.with_ctx(AuthenticatedCtx { user }))
                        }
                        None => Err(rspc::Error::new(
                            ErrorCode::Unauthorized,
                            "Unauthorized".into(),
                        )),
                    }
                })
            })
            .query("another", |t| {
                t(|_, _: ()| {
                    println!("ANOTHER QUERY");
                    "Another Result!"
                })
            })
            .subscription("subscriptions.pings", |t| {
                t(|_ctx, _args: ()| {
                    stream! {
                        println!("Client subscribed to 'pings'");
                        for i in 0..5 {
                            println!("Sending ping {}", i);
                            yield "ping".to_string();
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                })
            })
            // Reject all middleware
            .middleware(|mw| {
                mw.middleware(|_mw| async move {
                    Err(rspc::Error::new(
                        ErrorCode::Unauthorized,
                        "Unauthorized".into(),
                    )) as Result<MiddlewareContext<_>, _>
                })
            })
            // Plugin middleware // TODO: Coming soon!
            // .middleware(|mw| mw.openapi(OpenAPIConfig {}))
            .build()
            .arced();

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // Attach the rspc router to your axum router. The closure is used to generate the request context for each request.
        .nest(
            "/rspc",
            rspc_axum::endpoint(router, || UnauthenticatedContext {
                session_id: Some("abc".into()), // Change this line to control whether you are authenticated and can access the "another" query.
            }),
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
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
