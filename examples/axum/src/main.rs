use std::{
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
    pin::Pin,
    sync::{atomic::AtomicU16, Arc},
    task::{Context, Poll},
    time::Duration,
};

use async_stream::stream;
use axum::routing::get;
use futures::{Stream, StreamExt};
use rspc::{
    internal::{middleware::mw, rspc_core::IntoRouter, todo::SerdeJsonFormat},
    Executor, ExportConfig, Rspc,
};
use serde::Serialize;
use specta::Type;
use tokio::{net::TcpListener, sync::broadcast, time::sleep};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
struct Ctx {
    x_demo_header: Option<String>,
}

#[derive(thiserror::Error, Serialize, Type, Debug)]
#[error("{0}")]
struct Error(&'static str);

// TODO: Use `Ctx`
const R: Rspc<(), Error> = Rspc::new();

#[derive(thiserror::Error, serde::Serialize, specta::Type, Debug)]
pub enum MyCustomError {
    #[error("I am broke")]
    IAmBroke,
}

// /// TODO
// pub enum Demo {}

// impl ArgumentMapper for Demo {
//     type State = ();
//     type Input<T> = (String, T)
//     where
//         T: DeserializeOwned + Type + 'static;

//     fn map<T: Serialize + DeserializeOwned + Type + 'static>(
//         arg: Self::Input<T>,
//     ) -> (T, Self::State) {
//         (arg.1, ())
//     }
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = R
        .router()
        .procedure(
            "version",
            R.with(mw(|mw, ctx| async move {
                // let y = Mw::next();

                mw.next(ctx).map(|resp| async move {
                    println!("Client requested version '{}'", resp);
                    resp
                })
            }))
            // .with(mw(|mw, ctx| async move { mw.next(ctx) }))
            // .with(ArgMapper::<Demo>::new(|mw, ctx, _state| async move {
            //     mw.next(ctx)
            // }))
            .query(|_, _: ()| {
                info!("Client requested version");
                Ok(env!("CARGO_PKG_VERSION"))
            }),
        )
        // .procedure(
        //     "X-Demo-Header",
        //     R.query(|ctx, _: ()| Ok(ctx.x_demo_header.unwrap_or_else(|| "No header".to_string()))),
        // )
        // .procedure("echo", R.query(|_, v: String| Ok(v)))
        // .procedure("echo2", R.query(|_, v: String| async move { Ok(v) }))
        // .procedure(
        //     "error",
        //     R.query(|_, _: ()| Err(Error("Something went wrong")) as Result<String, _>),
        // )
        // .procedure(
        //     "error",
        //     R.mutation(|_, _: ()| Err(Error("Something went wrong")) as Result<String, _>),
        // )
        // .procedure(
        //     "transformMe",
        //     R.query(|_, _: ()| Ok("Hello, world!".to_string())),
        // )
        // .procedure(
        //     "sendMsg",
        //     R.mutation(|_, v: String| {
        //         println!("Client said '{}'", v);
        //         Ok(v)
        //     }),
        // )
        // .procedure(
        //     "pings",
        //     R.subscription(|_, _: ()| {
        //         println!("Client subscribed to 'pings'");
        //         stream! {
        //             yield Ok("start".to_string());
        //             for i in 0..5 {
        //                 info!("Sending ping {}", i);
        //                 yield Ok(i.to_string());
        //                 sleep(Duration::from_secs(1)).await;
        //             }
        //         }
        //     }),
        // )
        // .procedure(
        //     "errorPings",
        //     R.subscription(|_ctx, _args: ()| {
        //         stream! {
        //             for _ in 0..5 {
        //                 yield Ok("ping".to_string());
        //                 sleep(Duration::from_secs(1)).await;
        //             }
        //             yield Err(Error("Something went wrong"));
        //         }
        //     }),
        // )
        // .procedure(
        //     "testSubscriptionShutdown",
        //     R.subscription({
        //         static COUNT: AtomicU16 = AtomicU16::new(0);
        //         |_, _: ()| {
        //             let id = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //             pub struct HandleDrop {
        //                 id: u16,
        //                 sent: bool,
        //             }
        //             impl Stream for HandleDrop {
        //                 type Item = u16;
        //                 fn poll_next(
        //                     mut self: Pin<&mut Self>,
        //                     _: &mut Context<'_>,
        //                 ) -> Poll<Option<Self::Item>> {
        //                     if self.sent {
        //                         Poll::Ready(None)
        //                     } else {
        //                         self.sent = true;
        //                         Poll::Ready(Some(self.id))
        //                     }
        //                 }
        //             }
        //             impl Drop for HandleDrop {
        //                 fn drop(&mut self) {
        //                     println!("Dropped subscription with id {}", self.id);
        //                 }
        //             }
        //             HandleDrop { id, sent: false }.map(Ok)
        //         }
        //     }),
        // )
        // .procedure(
        //     "customErr",
        //     R.error::<MyCustomError>()
        //         .query(|_, _args: ()| Err::<(), _>(MyCustomError::IAmBroke)),
        // )
        // .procedure("batchingTest", {
        //     let (tx, _) = broadcast::channel(10);
        //     tokio::spawn({
        //         let tx = tx.clone();
        //         async move {
        //             let mut timer = tokio::time::interval(Duration::from_secs(1));
        //             loop {
        //                 timer.tick().await;
        //                 tx.send("ping".to_string()).ok();
        //             }
        //         }
        //     });
        //     R.subscription(move |_, _: ()| {
        //         let mut rx = tx.subscribe();
        //         stream! {
        //             while let Ok(msg) = rx.recv().await {
        //                 yield Ok(msg);
        //             }
        //         }
        //     })
        // })
        .build()
        .unwrap();

    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
        ))
        .unwrap();

    let e: Executor = router.build();

    println!("{:?}", e);

    // TODO: make a high-level rspc wrapper around the `Executor`
    // TODO: query vs mutation vs subscription????
    // TODO: Does the format really need to be `Arc`ed???
    let y = e.execute("version", Arc::new(SerdeJsonFormat {}));
    println!("{:?}", y);
    println!("{:?}", y.collect::<Vec<_>>().await);

    // e.execute(name, format)

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // .nest(
        //     "/rspc",
        //     rspc_axum::endpoint(router, || {
        //         // println!("Client requested operation '{}'", req.uri().path());
        //         //     Ctx {
        //         //         x_demo_header: req
        //         //             .headers()
        //         //             .get("X-Demo-Header")
        //         //             .map(|v| v.to_str().unwrap().to_string()),
        //         //     }
        //         Ctx {
        //             x_demo_header: None,
        //         }
        //     }),
        // )
        .layer(cors);

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000));
    println!("listening on http://{}/rspc", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
