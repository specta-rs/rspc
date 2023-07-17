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
    internal::exec::{
        AsyncRuntime, Connection, ConnectionTask, Executor, IncomingMessage, SubscriptionMap,
        TokioRuntime,
    },
    BuiltRouter,
};

// TODO: Move to https://tauri.app/v1/guides/features/plugin/#advanced -> This should help with avoiding cloning on shared state?

struct WindowManager<TCtxFn, TCtx, R>
where
    TCtx: Send + Sync + 'static,
    TCtxFn: Fn(Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
    R: AsyncRuntime,
{
    executor: Executor<TCtx, R>,
    ctx_fn: TCtxFn,
    windows: Mutex<HashMap<u64, Arc<Mutex<SubscriptionMap<TokioRuntime>>>>>,
}

impl<TCtxFn, TCtx, R> WindowManager<TCtxFn, TCtx, R>
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: Fn(Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
    R: AsyncRuntime,
{
    pub fn new(ctx_fn: TCtxFn, router: Arc<BuiltRouter<TCtx>>) -> Arc<Self> {
        Arc::new(Self {
            executor: Executor::new(router),
            ctx_fn,
            windows: Mutex::new(HashMap::new()),
        })
    }

    pub fn on_page_load(self: Arc<Self>, window: Window<tauri::Wry>) {
        let mut hasher = DefaultHasher::new();
        window.hash(&mut hasher);
        let window_hash = hasher.finish();

        let mut windows = self.windows.lock().unwrap();
        if let Some(subscriptions) = windows.get(&window_hash) {
            // Shutdown all subscriptions for the previously loaded page is there was one
            // Everything stays around though so we don't need to recreate it

            let mut subscriptions = subscriptions.lock().unwrap();
            for (_, handle) in subscriptions.drain() {
                TokioRuntime::cancel_task(handle);
            }
        } else {
            // Setup window for subscriptions

            let executor = self.executor.clone();
            let (tx, rx) = mpsc::unbounded_channel();
            let socket = Socket {
                recv: rx,
                window: window.clone(),
            };
            let ctx = (self.ctx_fn)(window.clone());
            let handle = R::spawn(async move {
                ConnectionTask::<R, TCtx, _, _>::new(Connection::new(ctx, executor), socket).await;
            });

            let subscriptions = Arc::new(Mutex::new(SubscriptionMap::<TokioRuntime>::default()));
            windows.insert(window_hash, subscriptions.clone());
            drop(windows);

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

    pub fn close_requested(&self, window: &Window<tauri::Wry>) {
        let mut hasher = DefaultHasher::new();
        window.hash(&mut hasher);
        let window_hash = hasher.finish();

        if let Some(rspc_window) = self.windows.lock().unwrap().remove(&window_hash) {
            TokioRuntime::spawn(async move {
                let mut subscriptions = rspc_window.lock().unwrap();
                for (_, tx) in subscriptions.drain() {
                    TokioRuntime::cancel_task(tx);
                }
            });
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
    let manager = WindowManager::<_, _, TokioRuntime>::new(ctx_fn, router);
    Builder::new("rspc")
        .on_page_load(move |window, _page| {
            manager.clone().on_page_load(window.clone());

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

impl futures::Sink<String> for Socket {
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl futures::Stream for Socket {
    type Item = Result<IncomingMessage, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.recv.poll_recv(cx).map(|v| v.map(Ok))
    }
}
