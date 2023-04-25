use std::{borrow::Cow, collections::HashMap, sync::Arc};

use futures::{SinkExt, StreamExt};
use futures_channel::mpsc;
use futures_locks::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime, WindowEvent,
};

use crate::{
    internal::jsonrpc::{self, handle_json_rpc, SubscriptionSender},
    Router,
};

pub fn plugin<R: Runtime, TCtx, TMeta>(
    router: Arc<Router<TCtx, TMeta>>,
    ctx_fn: impl Fn() -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<R>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
{
    let ctx_fn = Arc::new(ctx_fn);
    // let active_windows = Arc::new(Mutex::new(HashMap::new()));
    Builder::new("rspc")
        .on_page_load(move |window, _page| {
            let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
            let (resp_tx, mut resp_rx) = mpsc::channel(1024);
            let subscriptions = Arc::new(Mutex::new(HashMap::new()));

            tokio::spawn({
                let window = window.clone();
                // let shutdown_tx = shutdown_tx.clone();
                async move {
                    // {
                    //     let active_windows = active_windows.lock().await;
                    //     // The window previously had a page open but given a `page_reload` event has happened to trigger this we can shutdown all it's subscriptions
                    //     if let Some(shutdown_tx) = active_windows.get(&window.id()) {
                    //         shutdown_tx
                    //             .send(())
                    //             .await
                    //             .map_err(|_| {
                    //                 #[cfg(feature = "tracing")]
                    //                 tracing::error!("failed to emit shutdown signal");
                    //             })
                    //             .ok();
                    //     }

                    //     // Add the current window into the map
                    //     active_windows.lock().await.insert(window.id(), shutdown_tx); // TODO: If `window.id()` existed this could be a workaround
                    // }

                    loop {
                        tokio::select! {
                            biased;
                            _ = shutdown_rx.next() => {
                                println!("SHUTDOWN {:?}", _page.url()); // TODO
                                break;
                            }
                            res = resp_rx.next() => {
                                if let Some(res) = res {
                                    window
                                        .emit("plugin:rspc:transport:resp", res)
                                        .map_err(|err| {
                                            #[cfg(feature = "tracing")]
                                            tracing::error!("failed to emit JSON-RPC response: {}", err);
                                        })
                                        .ok();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            });

            // TODO: Clear subscriptions on page reload also

            #[allow(clippy::single_match)]
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { .. } => {
                    println!("Closing window"); // TODO
                    let mut shutdown_tx = shutdown_tx.clone();
                    tokio::spawn(async move {
                        shutdown_tx
                            .send(())
                            .await
                            .map_err(|_| {
                                #[cfg(feature = "tracing")]
                                tracing::error!("failed to emit shutdown signal");
                            })
                            .ok();
                    });
                },
                // WindowEvent::Reload {
                //     // This would be the same as `WindowEvent::CloseRequested` and would save me needing to keep track of the windows in a map like also shown above
                // }
                _ => {}
            });

            let router = router.clone();
            let ctx_fn = ctx_fn.clone();
            window.listen("plugin:rspc:transport", move |event| {
                let reqs = match event.payload() {
                    Some(v) => match serde_json::from_str::<serde_json::Value>(v).and_then(|v| if v.is_array() {
                        serde_json::from_value::<Vec<jsonrpc::Request>>(v)
                    } else {
                       serde_json::from_value::<jsonrpc::Request>(v).map(|v| vec![v])
                    }) {
                        Ok(v) => v,
                        Err(err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("failed to parse JSON-RPC request: {}", err);
                            return;
                        }
                    },
                    None => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Tauri event payload is empty");

                        return;
                    }
                };

                for req in reqs {
                    let ctx = ctx_fn();
                    let mut resp_tx = resp_tx.clone();
                    let subscriptions = subscriptions.clone();
                    let router = router.clone();
    
                    tokio::spawn(async move {
                        // When the thread which holds the receiver for `resp_rx` is dropped it will cause this thread to be shutdown.
                        handle_json_rpc(
                            ctx,
                            req,
                            Cow::Owned(router),
                            SubscriptionSender(&mut resp_tx, subscriptions),
                        )
                        .await;
                    });
                }
            });
        })
        .build()
}
