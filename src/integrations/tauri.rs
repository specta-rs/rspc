//! Access rspc via the Tauri IPC bridge.

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    convert::Infallible,
    hash::{Hash, Hasher},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Window, WindowEvent,
};
use tauri_specta::Event;
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    internal::exec::{
        run_connection, AsyncRuntime, ConnectionTask, IncomingMessage, Response, TokioRuntime,
    },
    Router,
};

#[derive(Clone, Debug, serde::Deserialize, specta::Type, tauri_specta::Event)]
struct Msg(serde_json::Value);

struct WindowManager<TCtxFn, TCtx>
where
    TCtx: Send + Sync + 'static,
    TCtxFn: Fn(&Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
{
    router: Arc<Router<TCtx>>,
    ctx_fn: TCtxFn,
    windows: Mutex<HashMap<u64, mpsc::UnboundedSender<()>>>,
}

impl<TCtxFn, TCtx> WindowManager<TCtxFn, TCtx>
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: Fn(&Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
{
    pub fn new(ctx_fn: TCtxFn, router: Arc<Router<TCtx>>) -> Arc<Self> {
        Arc::new(Self {
            router,
            ctx_fn,
            windows: Mutex::new(HashMap::new()),
        })
    }

    pub fn on_page_load<R: AsyncRuntime>(self: Arc<Self>, window: Window<tauri::Wry>) {
        let mut hasher = DefaultHasher::new();
        window.hash(&mut hasher);
        let window_hash = hasher.finish();

        #[allow(clippy::unwrap_used)] // TODO: Stop using unwrap
        let mut windows = self.windows.lock().unwrap();
        if let Some(shutdown_streams_tx) = windows.get(&window_hash) {
            // Shutdown all subscriptions for the previously loaded page is there was one
            // All the previous threads and stuff stays around though so we don't need to recreate it

            shutdown_streams_tx.send(()).ok();
        } else {
            let (clear_subscriptions_tx, mut clear_subscriptions_rx) = mpsc::unbounded_channel();
            windows.insert(window_hash, clear_subscriptions_tx);
            drop(windows);

            let (tx, rx) = mpsc::unbounded_channel();

            // tauri::async_runtime::spawn(ConnectionTask::<R, _, _, _>::new(
            //     (self.ctx_fn)(&window),
            //     self.router.clone(),
            //     Socket {
            //         recv: rx,
            //         window: window.clone(),
            //     },
            //     Some(Box::new(move |cx| clear_subscriptions_rx.poll_recv(cx))),
            // ));

            let (_, rrx) = futures::channel::oneshot::channel();

            tauri::async_runtime::spawn(run_connection::<R, _, _, _>(
                (self.ctx_fn)(&window),
                self.router.clone(),
                Socket {
                    recv: rx,
                    window: window.clone(),
                },
                Some(rrx),
            ));

            Msg::listen(&window, move |event| {
                tx.send(IncomingMessage::Msg(Ok(event.payload.0))).ok();
            });
        }
    }

    #[allow(clippy::unwrap_used)] // TODO: Stop using unwrap
    pub fn close_requested(&self, window: &Window<tauri::Wry>) {
        let mut hasher = DefaultHasher::new();
        window.hash(&mut hasher);
        let window_hash = hasher.finish();

        if let Some(shutdown_streams_tx) = self.windows.lock().unwrap().get(&window_hash) {
            shutdown_streams_tx.send(()).ok();
        }
    }
}

const PLUGIN_NAME: &str = "rspc";

macro_rules! specta_builder {
    () => {
        tauri_specta::ts::builder().events(tauri_specta::collect_events![Msg, TransportResp])
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn export_types() {
        specta_builder!()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./packages/tauri/src/types.ts"))
            .export_for_plugin(PLUGIN_NAME)
            .unwrap();
    }
}

pub fn plugin<TCtx>(
    router: Arc<Router<TCtx>>,
    ctx_fn: impl Fn(&Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<tauri::Wry>
where
    TCtx: Clone + Send + Sync + 'static,
{
    let manager = WindowManager::new(ctx_fn, router);

    let plugin_utils = specta_builder!().into_plugin_utils(PLUGIN_NAME);

    Builder::new(PLUGIN_NAME)
        .invoke_handler(plugin_utils.invoke_handler)
        .setup(|app| {
            (plugin_utils.setup)(app);
            Ok(())
        })
        .on_page_load(move |window, _page| {
            manager.clone().on_page_load::<TokioRuntime>(window.clone());

            window.on_window_event({
                let window = window.clone();
                let manager = manager.clone();
                move |event| {
                    if let WindowEvent::CloseRequested { .. } = event {
                        manager.close_requested(&window);
                    }
                }
            })
        })
        .build()
}

struct Socket {
    // TODO: Bounded channel?
    recv: mpsc::UnboundedReceiver<IncomingMessage>,
    window: Window,
}

#[derive(Clone, serde::Serialize, specta::Type, tauri_specta::Event)]
#[specta(inline)]
struct TransportResp(Vec<Response>);

// TODO: Can we use utils in `futures` to remove this impl?
impl futures::Sink<Vec<Response>> for Socket {
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Vec<Response>) -> Result<(), Self::Error> {
        TransportResp(item)
            .emit(&self.window)
            .map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("failed to emit JSON-RPC response: {}", _err);
            })
            .ok();

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

// TODO: Can we use utils in `futures` to remove this impl?
impl futures::Stream for Socket {
    type Item = Result<IncomingMessage, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.recv.poll_recv(cx).map(|v| v.map(Ok))
    }
}
