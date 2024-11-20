//! rspc-tauri: Tauri integration for [rspc](https://rspc.dev).
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Emitter, Listener, Manager, Runtime,
};
use tokio::sync::{mpsc, Mutex};

use rspc::{
    internal::jsonrpc::{self, handle_json_rpc, Sender, SubscriptionMap},
    Router,
};

pub fn plugin<R: Runtime, TCtx, TMeta>(
    router: Arc<Router<TCtx, TMeta>>,
    ctx_fn: impl Fn(AppHandle<R>) -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<R>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
{
    Builder::new("rspc")
        .setup(|app_handle, _| {
            let (tx, mut rx) = mpsc::unbounded_channel::<jsonrpc::Request>();
            let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<jsonrpc::Response>();
            // TODO: Don't keep using a tokio mutex. We don't need to hold it over the await point.
            let subscriptions = Arc::new(Mutex::new(HashMap::new()));

            tokio::spawn({
                let app_handle = app_handle.clone();
                async move {
                    while let Some(req) = rx.recv().await {
                        let ctx = ctx_fn(app_handle.clone());
                        let router = router.clone();
                        let mut resp_tx = resp_tx.clone();
                        let subscriptions = subscriptions.clone();
                        tokio::spawn(async move {
                            handle_json_rpc(
                                ctx,
                                req,
                                &router,
                                &mut Sender::ResponseChannel(&mut resp_tx),
                                &mut SubscriptionMap::Mutex(subscriptions.borrow()),
                            )
                            .await;
                        });
                    }
                }
            });

            {
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    while let Some(event) = resp_rx.recv().await {
                        let _ = app_handle
                            .emit("plugin:rspc:transport:resp", event)
                            .map_err(|err| {
                                #[cfg(feature = "tracing")]
                                tracing::error!("failed to emit JSON-RPC response: {}", err);
                            });
                    }
                });
            }

            app_handle.listen_any("plugin:rspc:transport", move |event| {
                let _ = tx
                    .send(match serde_json::from_str(event.payload()) {
                        Ok(v) => v,
                        Err(err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("failed to parse JSON-RPC request: {}", err);
                            return;
                        }
                    })
                    .map_err(|err| {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to send JSON-RPC request: {}", err);
                    });
            });

            Ok(())
        })
        .build()
}
