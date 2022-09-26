use std::path::PathBuf;

use rspc::{Config, ErrorCode, Middleware, MiddlewareContext, Router};

#[derive(Debug, Clone)]
pub struct UnauthenticatedContext {
    pub session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct User {
    name: String,
}

async fn db_get_user_from_session(session_id: &str) -> User {
    User {
        name: "Monty Beaumont".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedCtx {
    user: User,
}

#[tokio::main]
async fn main() {
    let _r =
        Router::<UnauthenticatedContext>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
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
            .query("version", |t| {
                t(|_ctx, _: ()| {
                    println!("ANOTHER QUERY");
                    env!("CARGO_PKG_VERSION")
                })
            })
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
            .query("another", |t| {
                t(|_, _: ()| {
                    println!("ANOTHER QUERY");
                    "Another Result!"
                })
            })
            // Reject all middleware
            .middleware(|mw| {
                mw.middleware(|mw| async move {
                    Err(rspc::Error::new(
                        ErrorCode::Unauthorized,
                        "Unauthorized".into(),
                    )) as Result<MiddlewareContext<_>, _>
                })
            })
            // Plugin middleware
            // .middleware(|mw| mw.openapi(OpenAPIConfig {}))
            .build();
}
