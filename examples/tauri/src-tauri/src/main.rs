// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    pin::Pin,
    sync::atomic::AtomicU16,
    task::{Context, Poll},
    time::Duration,
};

use async_stream::stream;
use futures::Stream;
use rspc::{ErrorCode, Rspc};
use tokio::time::sleep;

#[derive(Clone)]
struct Ctx {
    x_demo_header: Option<String>,
}

#[derive(thiserror::Error, serde::Serialize, specta::Type, Debug)]
#[error("{0}")]
struct Error(&'static str);

const R: Rspc<Ctx, Error> = Rspc::new();

#[tokio::main]
async fn main() {
    let router = R
        .router()
        .procedure(
            "version",
            R.with(|mw, ctx| async move {
                mw.next(ctx).map(|resp| async move {
                    println!("Client requested version '{}'", resp);
                    resp
                })
            })
            .with(|mw, ctx| async move { mw.next(ctx) })
            .query(|_, _: ()| Ok(env!("CARGO_PKG_VERSION"))),
        )
        .procedure(
            "X-Demo-Header",
            R.query(|ctx, _: ()| Ok(ctx.x_demo_header.unwrap_or_else(|| "No header".to_string()))),
        )
        .procedure("echo", R.query(|_, v: String| Ok(v)))
        .procedure(
            "error",
            R.query(|_, _: ()| {
                Err::<String, _>(Error("Something went wrong"))
            }),
        )
        .procedure(
            "error",
            R.mutation(|_, _: ()| {
                Err::<String, _>(Error("Something went wrong"))
            }),
        )
        .procedure(
            "transformMe",
            R.query(|_, _: ()| Ok("Hello, world!".to_string())),
        )
        .procedure(
            "sendMsg",
            R.mutation(|_, v: String| {
                println!("Client said '{}'", v);
                Ok(v)
            }),
        )
        .procedure(
            "pings",
            R.subscription(|_, _: ()| {
                println!("Client subscribed to 'pings'");
                stream! {
                    yield "start".to_string();
                    for i in 0..5 {
                        println!("Sending ping {}", i);
                        yield i.to_string();
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }),
        )
        .procedure("errorPings", R.subscription(|_ctx, _args: ()| {
            stream! {
                for _ in 0..5 {
                    yield Ok("ping".to_string());
                    sleep(Duration::from_secs(1)).await;
                }
                yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
            }
        }))
        .procedure(
            "testSubscriptionShutdown",
            R.subscription({
                static COUNT: AtomicU16 = AtomicU16::new(0);
                |_, _: ()| {
                    let id = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    pub struct HandleDrop {
                        id: u16,
                        send: bool,
                    }

                    impl Stream for HandleDrop {
                        type Item = u16;

                        fn poll_next(
                            mut self: Pin<&mut Self>,
                            _: &mut Context<'_>,
                        ) -> Poll<Option<Self::Item>> {
                            if self.send {
                                Poll::Pending
                            } else {
                                self.send = true;
                                Poll::Ready(Some(self.id))
                            }
                        }
                    }

                    impl Drop for HandleDrop {
                        fn drop(&mut self) {
                            println!("Dropped subscription with id {}", self.id);
                        }
                    }

                    HandleDrop { id, send: false }
                }
            }),
        )
        .build()
        .unwrap()
        .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router, |_| Ctx {
            x_demo_header: None,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
