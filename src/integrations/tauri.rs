//! Access rspc via the Tauri IPC bridge.

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    convert::Infallible,
    hash::{Hash, Hasher},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use serde_json::Value;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Window, WindowEvent,
};
use tokio::sync::mpsc;

use crate::{
    internal::exec::{AsyncRuntime, ConnectionTask, Executor, IncomingMessage, TokioRuntime},
    BuiltRouter,
};

struct WindowManager<TCtxFn, TCtx>
where
    TCtx: Send + Sync + 'static,
    TCtxFn: Fn(Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
{
    executor: Executor<TCtx>,
    ctx_fn: TCtxFn,
    windows: Mutex<HashMap<u64, mpsc::UnboundedSender<()>>>,
}

impl<TCtxFn, TCtx> WindowManager<TCtxFn, TCtx>
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: Fn(Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
{
    pub fn new(ctx_fn: TCtxFn, router: Arc<BuiltRouter<TCtx>>) -> Arc<Self> {
        Arc::new(Self {
            executor: Executor::new(router),
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
            let (clear_subscriptions_tx, clear_subscriptions_rx) = mpsc::unbounded_channel();
            windows.insert(window_hash, clear_subscriptions_tx);
            drop(windows);

            let (tx, rx) = mpsc::unbounded_channel();
            R::spawn(ConnectionTask::<R, _, _, _>::new(
                (self.ctx_fn)(window.clone()),
                self.executor.clone(),
                Socket {
                    recv: rx,
                    window: window.clone(),
                },
                Some(clear_subscriptions_rx),
            ));

            window.listen("plugin:rspc:transport", move |event| {
                let Some(payload) = event.payload() else {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Tauri event payload is empty");

                        return;
                    };

                // God damn, Tauri is cringe. Why do they string double encode the payload.
                let payload = match serde_json::from_str::<serde_json::Value>(payload) {
                    Ok(v) => match v {
                        Value::String(s) => serde_json::from_str::<serde_json::Value>(&s),
                        v => Ok(v),
                    },
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to parse JSON-RPC request: {}", err);
                        return;
                    }
                };

                tx.send(IncomingMessage::Msg(payload)).ok();
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

pub fn plugin<TCtx>(
    router: Arc<BuiltRouter<TCtx>>,
    ctx_fn: impl Fn(Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<tauri::Wry>
where
    TCtx: Clone + Send + Sync + 'static,
{
    let manager = WindowManager::new(ctx_fn, router);
    Builder::new("rspc")
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

// TODO: Can we use utils in `futures` to remove this impl?
impl futures::Sink<String> for Socket {
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        self.window
            .emit("plugin:rspc:transport:resp", item)
            .map_err(|err| {
                #[cfg(feature = "tracing")]
                tracing::error!("failed to emit JSON-RPC response: {}", err);
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
