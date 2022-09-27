use std::{collections::HashMap, sync::Arc};

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
                        app_handle
                            .emit_all("plugin:rspc:transport:resp", event)
                            .unwrap();
                    }
                });
            }

            app_handle.listen_global("plugin:rspc:transport", move |event| {
                tx.send(serde_json::from_str(event.payload().unwrap()).unwrap())
                    .unwrap();
            });

            Ok(())
        })
        .build()
}
