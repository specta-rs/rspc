use std::sync::Arc;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::mpsc;

use crate::{Request, Response, Router};

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
            let (tx, mut rx) = mpsc::unbounded_channel::<Request>();
            let (resp_tx, mut resp_rx) = mpsc::unbounded_channel::<Response>();

            let app_handle2 = app_handle.clone();
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    let result = event.handle(ctx_fn(), &router, Some(&resp_tx)).await;

                    if !matches!(result, Response::None) {
                        app_handle2
                            .emit_all("plugin:rspc:transport:resp", result)
                            .unwrap();
                    }
                }
            });

            let app_handle3 = app_handle.clone();
            tokio::spawn(async move {
                while let Some(event) = resp_rx.recv().await {
                    app_handle3
                        .emit_all("plugin:rspc:transport:resp", event)
                        .unwrap();
                }
            });

            app_handle.listen_global("plugin:rspc:transport", move |event| {
                tx.send(serde_json::from_str(event.payload().unwrap()).unwrap())
                    .unwrap();
            });

            Ok(())
        })
        .build()
}
