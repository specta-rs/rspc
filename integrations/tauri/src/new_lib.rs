//! rspc-tauri: Tauri integration for [rspc](https://rspc.dev).
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

use std::{borrow::Borrow, collections::HashMap, future::Future, pin::Pin, sync::Arc};

use jsonrpc::RequestId;
use jsonrpc_exec::{handle_json_rpc, Sender, SubscriptionMap};
use rspc_core::Procedures;
use tauri::{
    async_runtime::{spawn, Mutex},
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager,
};
use tokio::sync::oneshot;

mod jsonrpc;
mod jsonrpc_exec;

struct State<R, TCtxFn, TCtx> {
    subscriptions: Arc<Mutex<HashMap<RequestId, oneshot::Sender<()>>>>,
    ctx_fn: TCtxFn,
    procedures: Arc<Procedures<TCtx>>,
    phantom: std::marker::PhantomData<R>,
}

impl<R, TCtxFn, TCtx> State<R, TCtxFn, TCtx>
where
    R: tauri::Runtime,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    fn new(procedures: Procedures<TCtx>, ctx_fn: TCtxFn) -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            ctx_fn,
            procedures: Arc::new(procedures),
            phantom: Default::default(),
        }
    }
}

trait HandleRpc<R: tauri::Runtime>: Send + Sync {
    fn handle_rpc(
        &self,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<jsonrpc::Response>,
        req: jsonrpc::Request,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<R, TCtxFn, TCtx> HandleRpc<R> for State<R, TCtxFn, TCtx>
where
    R: tauri::Runtime + Send + Sync,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    fn handle_rpc(
        &self,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<jsonrpc::Response>,
        req: jsonrpc::Request,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let ctx = (self.ctx_fn)(window);
        let procedures = self.procedures.clone();
        let subscriptions = self.subscriptions.clone();

        let (mut resp_tx, mut resp_rx) =
            tokio::sync::mpsc::unbounded_channel::<jsonrpc::Response>();

        spawn(async move {
            while let Some(resp) = resp_rx.recv().await {
                channel.send(resp).ok();
            }
        });

        Box::pin(async move {
            handle_json_rpc(
                ctx,
                req,
                &procedures,
                &mut Sender::ResponseChannel(&mut resp_tx),
                &mut SubscriptionMap::Mutex(subscriptions.borrow()),
            )
            .await;
        })
    }
}

type DynState<R> = Arc<dyn HandleRpc<R>>;

#[tauri::command]
async fn handle_rpc<R: tauri::Runtime>(
    state: tauri::State<'_, DynState<R>>,
    window: tauri::Window<R>,
    channel: tauri::ipc::Channel<jsonrpc::Response>,
    req: jsonrpc::Request,
) -> Result<(), ()> {
    state.handle_rpc(window, channel, req).await;

    Ok(())
}

pub fn plugin<R, TCtxFn, TCtx>(
    routes: impl Into<Procedures<TCtx>>,
    ctx_fn: TCtxFn,
) -> TauriPlugin<R>
where
    R: tauri::Runtime + Send + Sync,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
{
    let routes = routes.into();

    Builder::new("rspc")
        .invoke_handler(generate_handler![handle_rpc])
        .setup(move |app_handle, _| {
            app_handle.manage(Arc::new(State::new(routes, ctx_fn)) as DynState<R>);

            Ok(())
        })
        .build()
}
