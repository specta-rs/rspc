use std::{marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use async_stream::stream;
use axum::{http::request::Parts, routing::get};
use rspc::{
    middleware::Middleware, Error2, Procedure2, ProcedureBuilder, ResolverInput, ResolverOutput,
    Router2,
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

// `Clone` is only required for usage with Websockets
#[derive(Clone)]
struct Ctx {}

#[derive(Serialize, Type)]
pub struct MyCustomType(String);

#[derive(Type, Serialize)]
#[serde(tag = "type")]
#[specta(export = false)]
pub enum DeserializationError {
    // Is not a map-type so invalid.
    A(String),
}

// http://[::]:4000/rspc/version
// http://[::]:4000/legacy/version

// http://[::]:4000/rspc/nested.hello
// http://[::]:4000/legacy/nested.hello

// http://[::]:4000/rspc/error
// http://[::]:4000/legacy/error

// http://[::]:4000/rspc/echo
// http://[::]:4000/legacy/echo

// http://[::]:4000/rspc/echo?input=42
// http://[::]:4000/legacy/echo?input=42

fn mount() -> rspc::Router<Ctx> {
    let inner = rspc::Router::<Ctx>::new().query("hello", |t| t(|_, _: ()| "Hello World!"));

    let router = rspc::Router::<Ctx>::new()
        .merge("nested.", inner)
        .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        // .mutation("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .query("echo", |t| t(|_, v: String| v))
        .query("error", |t| {
            t(|_, _: ()| {
                Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    "Something went wrong".into(),
                )) as Result<String, rspc::Error>
            })
        })
        .query("transformMe", |t| t(|_, _: ()| "Hello, world!".to_string()))
        .mutation("sendMsg", |t| {
            t(|_, v: String| {
                println!("Client said '{}'", v);
                v
            })
        })
        // .mutation("anotherOne", |t| t(|_, v: String| Ok(MyCustomType(v))))
        .subscription("pings", |t| {
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
        // TODO: Results being returned from subscriptions
        // .subscription("errorPings", |t| t(|_ctx, _args: ()| {
        //     stream! {
        //         for i in 0..5 {
        //             yield Ok("ping".to_string());
        //             sleep(Duration::from_secs(1)).await;
        //         }
        //         yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
        //     }
        // }))
        .build();

    router
}

#[derive(Debug, Error, Serialize, Type)]
pub enum Error {
    #[error("you made a mistake: {0}")]
    Mistake(String),
}

impl Error2 for Error {}

pub struct BaseProcedure<TErr = Error>(PhantomData<TErr>);
impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>() -> ProcedureBuilder<TErr, Ctx, Ctx, TInput, TResult>
    where
        TErr: Error2,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure2::builder() // You add default middleware here
    }
}

fn test_unstable_stuff(router: Router2<Ctx>) -> Router2<Ctx> {
    router
        .procedure("newstuff", {
            <BaseProcedure>::builder().query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
        })
        .procedure("newstuff2", {
            <BaseProcedure>::builder()
                // .with(invalidation(|ctx: Ctx, key, event| false))
                .with(Middleware::new(
                    move |ctx: Ctx, input: (), next| async move {
                        let result = next.exec(ctx, input).await;
                        result
                    },
                ))
                .query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
        })
}

#[derive(Debug, Clone, Serialize, Type)]
pub enum InvalidateEvent {
    InvalidateKey(String),
}

fn invalidation<TError, TCtx, TInput, TResult>(
    handler: impl Fn(TCtx, TInput, InvalidateEvent) -> bool + Send + Sync + 'static,
) -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Clone + Send + 'static,
    TInput: Clone + Send + 'static,
    TResult: Send + 'static,
{
    let handler = Arc::new(handler);
    Middleware::new(move |ctx: TCtx, input: TInput, next| {
        let handler = handler.clone();
        async move {
            // TODO: Register this with `TCtx`
            let ctx2 = ctx.clone();
            let input2 = input.clone();
            let result = next.exec(ctx, input).await;

            // TODO: Unregister this with `TCtx`
            result
        }
    })
}

#[tokio::main]
async fn main() {
    let router = Router2::from(mount());
    let router = test_unstable_stuff(router);
    let (routes, types) = router.build().unwrap();

    rspc::Typescript::default()
        // .formatter(specta_typescript::formatter::prettier),
        .header("// My custom header")
        // .enable_source_maps() // TODO: Fix this
        .export_to(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
            &types,
        )
        .unwrap();

    // Be aware this is very experimental and doesn't support many types yet.
    // rspc::Rust::default()
    //     // .header("// My custom header")
    //     .export_to(
    //         PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../client/src/bindings.rs"),
    //         &types,
    //     )
    //     .unwrap();

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(routes, |parts: Parts| {
                println!("Client requested operation '{}'", parts.uri.path());
                Ctx {}
            }),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
