use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::{mpsc, Mutex};

use crate::{
    internal::jsonrpc::{self, handle_json_rpc, Sender, SubscriptionMap},
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
    Builder::new("rspc")
        .setup(|app_handle| {
            let (tx, mut rx) = mpsc::unbounded_channel::<jsonrpc::Request>();
            let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<jsonrpc::Response>();
            // TODO: Don't keep using a tokio mutex. We don't need to hold it over the await point.
            let subscriptions = Arc::new(Mutex::new(HashMap::new()));

            tokio::spawn(async move {
                while let Some(req) = rx.recv().await {
                    let ctx = ctx_fn();
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
            });

            {
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    while let Some(event) = resp_rx.recv().await {
                        let _ = app_handle
                            .emit_all("plugin:rspc:transport:resp", event)
                            .map_err(|err| {
                                #[cfg(feature = "tracing")]
                                tracing::error!("failed to emit JSON-RPC response: {}", err);
                            });
                    }
                });
            }

            app_handle.listen_global("plugin:rspc:transport", move |event| {
                let _ = tx
                    .send(
                        match serde_json::from_str(match event.payload() {
                            Some(v) => v,
                            None => {
                                #[cfg(feature = "tracing")]
                                tracing::error!("Tauri event payload is empty");

                                return;
                            }
                        }) {
                            Ok(v) => v,
                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!("failed to parse JSON-RPC request: {}", err);
                                return;
                            }
                        },
                    )
                    .map_err(|err| {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to send JSON-RPC request: {}", err);
                    });
            });

            Ok(())
        })
        .build()
}
