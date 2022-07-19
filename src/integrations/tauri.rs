use std::sync::Arc;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::mpsc;

use crate::{utils::Request, KeyDefinition, Router};

pub fn plugin<R: Runtime, TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>(
    router: Arc<Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>>,
    ctx_fn: impl Fn() -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<R>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    Builder::new("rspc")
        .setup(|app_handle| {
            let (tx, mut rx) = mpsc::unbounded_channel::<Request>();

            let app_handle2 = app_handle.clone();
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    app_handle2
                        .emit_all(
                            "plugin:rspc:transport:resp",
                            event.handle(ctx_fn(), &router).await,
                        )
                        .unwrap();
                }
            });

            app_handle.listen_global("plugin:rspc:transport", move |event| {
                tx.send(serde_json::from_str(event.payload().unwrap()).unwrap());
            });

            Ok(())
        })
        .build()
}
