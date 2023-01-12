use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::routing::get;
use rspc::{internal::MiddlewareState, Config, ErrorCode, Router};
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
            // .query("version", {
            //     #[derive(Serialize, rspc::Type)]
            //     struct Bruh<T> {
            //         meta: String,
            //         data: T,
            //     }
            //     // pub trait Normalize {}
            //     // #[derive(Object)]
            //     // pub struct Wrapper {
            //     //     pub scalar: i32,
            //     //     #[ref]
            //     //     pub ref_this: String,
            //     //     #[ref]
            //     //     pub wrapper_2: Wrapper2,
            //     //     pub vec_ref: Vec<AnotherObject>
            //     // }
            //     // #[derive(Object)]
            //     // pub struct Wrapper2 {
            //     //     #[ref]
            //     //     pub vec_ref: Vec<AnotherObject>
            //     // };
            //     fn typed<TResolver, TLayerCtx, TArg, TResolverMarker, TResultMarker>(
            //         builder: BuiltProcedureBuilder<TResolver>,
            //     ) -> BuiltProcedureBuilder<
            //         impl RequestResolver<
            //             TLayerCtx,
            //             DoubleArgMarker<TArg, FutureMarker<ResultMarker>>,
            //             FutureMarker<ResultMarker>,
            //             Arg = TArg,
            //         >,
            //     >
            //     where
            //         TArg: Type + DeserializeOwned,
            //         TLayerCtx: Send + Sync + 'static,
            //         TResolver: RequestResolver<TLayerCtx, TResolverMarker, TResultMarker, Arg = TArg>,
            //     {
            //         BuiltProcedureBuilder {
            //             resolver: move |ctx, arg| {
            //                 let val = builder.resolver.exec(ctx, arg);
            //                 async {
            //                     Ok(Bruh {
            //                         meta: "Bruh".to_string(),
            //                         data: val?.exec().await?,
            //                     })
            //                 }
            //             },
            //         }
            //     }
            //     |t| {
            //         t(|_ctx, _: ()| {
            //             println!("ANOTHER QUERY");
            //             env!("CARGO_PKG_VERSION")
            //         })
            //         .map(typed)
            //         .map(|_| BuiltProcedureBuilder {
            //             resolver: |_, _| {
            //                 println!("This resolver has been overwritten to an int!");
            //                 0
            //             },
            //         })
            //     }
            // })
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
                    )) as Result<MiddlewareState<_>, _>
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
            router
                .endpoint(|| UnauthenticatedContext {
                    session_id: Some("abc".into()), // Change this line to control whether you are authenticated and can access the "another" query.
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
