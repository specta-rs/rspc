use std::{collections::HashMap, sync::Arc};

use serde::de::Error;
use serde_json::Value;
use tauri::{
    async_runtime::Mutex,
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::mpsc;

use crate::{
    internal::jsonrpc::{self, handle_json_rpc, Sender, SubscriptionMap},
    Router,
};

pub fn plugin<R: Runtime, TCtx>(
    router: Arc<Router<TCtx>>,
    ctx_fn: impl Fn() -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<R>
where
    TCtx: Send + 'static,
{
    Builder::new("rspc")
        .setup(|app_handle| {
            let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<jsonrpc::Response>();
            let subscriptions = Arc::new(Mutex::new(HashMap::new()));

            {
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    while let Some(event) = resp_rx.recv().await {
                        let _ = app_handle
                            .emit_all("plugin:rspc:transport:resp", event)
                            .map_err(|_err| {
                                #[cfg(feature = "tracing")]
                                tracing::error!("failed to emit JSON-RPC response: {}", _err);
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
                .and_then(|v| match v {
                    // TODO: This is a temporary hack for: https://github.com/oscartbeaumont/rspc/issues/77
                    Value::String(v) => serde_json::from_str::<Value>(&v),
                    _ => Ok(v),
                })
                .and_then(|v| match v {
                    Value::Object(v) => {
                        serde_json::from_value::<jsonrpc::Request>(Value::Object(v))
                            .map(|v| vec![v])
                    }
                    Value::Array(v) => {
                        serde_json::from_value::<Vec<jsonrpc::Request>>(Value::Array(v))
                    }
                    _ => Err(serde_json::Error::custom("invalid JSON-RPC request")),
                }) {
                    Ok(v) => v,
                    Err(_err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to decode JSON-RPC request: {}", _err);
                        return;
                    }
                };

                for req in req {
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
                            &mut SubscriptionMap::Mutex(&subscriptions),
                        )
                        .await;
                    });
                }
            });

            Ok(())
        })
        .build()
}
