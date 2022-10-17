use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::mpsc;

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
            let (mut resp_tx, mut resp_rx) = mpsc::unbounded_channel::<jsonrpc::Response>();
            let mut subscriptions = HashMap::new();

            tokio::spawn(async move {
                while let Some(req) = rx.recv().await {
                    handle_json_rpc(
                        ctx_fn(),
                        req,
                        &router,
                        &mut Sender::ResponseChannel(&mut resp_tx),
                        &mut SubscriptionMap::Ref(&mut subscriptions),
                    )
                    .await;
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
                let req = match serde_json::from_str::<Value>(match event.payload() {
                    Some(v) => v,
                    None => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Tauri event payload is empty");

                        return;
                    }
                })
                .and_then(|v| match v.is_array() {
                    true => serde_json::from_value::<Vec<jsonrpc::Request>>(v),
                    false => serde_json::from_value::<jsonrpc::Request>(v).map(|v| vec![v]),
                }) {
                    Ok(v) => v,
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to decode JSON-RPC request: {}", err);
                        return;
                    }
                };

                for req in req {
                    let _ = tx.send(req).map_err(|err| {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to send JSON-RPC request: {}", err);
                    });
                }
            });

            Ok(())
        })
        .build()
}
