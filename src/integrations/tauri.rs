//! Access rspc via the Tauri IPC bridge.

use std::{
    borrow::Cow,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, MutexGuard},
};

use serde_json::Value;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime, Window, WindowEvent,
};

use crate::{
    internal::exec::{
        self, AsyncRuntime, Executor, OwnedStream, SubscriptionManager, SubscriptionMap,
        TokioRuntime,
    },
    CompiledRouter,
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
    pub fn new(ctx_fn: TCtxFn, router: Arc<CompiledRouter<TCtx>>) -> Arc<Self> {
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
        // Shutdown all subscriptions for the previously loaded page is there was one
        if let Some(subscriptions) = windows.get(&window_hash) {
            let mut subscriptions = subscriptions.lock().unwrap();
            for (_, handle) in subscriptions.drain() {
                TokioRuntime::cancel_task(handle);
            }
        } else {
            let subscriptions = Arc::new(Mutex::new(SubscriptionMap::<TokioRuntime>::default()));
            windows.insert(window_hash, subscriptions.clone());
            drop(windows);

            window.listen("plugin:rspc:transport", {
                let window = window.clone();
                move |event| {
                    let reqs = match event.payload() {
                        Some(v) => {
                            let v = match serde_json::from_str::<serde_json::Value>(v) {
                                Ok(v) => match v {
                                    Value::String(s) => {
                                        match serde_json::from_str::<serde_json::Value>(&s) {
                                            Ok(v) => v,
                                            Err(err) => {
                                                #[cfg(feature = "tracing")]
                                                tracing::error!(
                                                    "failed to parse JSON-RPC request: {}",
                                                    err
                                                );
                                                return;
                                            }
                                        }
                                    }
                                    v => v,
                                },
                                Err(err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("failed to parse JSON-RPC request: {}", err);
                                    return;
                                }
                            };

                            match if v.is_array() {
                                serde_json::from_value::<Vec<exec::Request>>(v)
                            } else {
                                serde_json::from_value::<exec::Request>(v).map(|v| vec![v])
                            } {
                                Ok(v) => v,
                                Err(err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("failed to parse JSON-RPC request: {}", err);
                                    return;
                                }
                            }
                        }
                        None => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Tauri event payload is empty");

                            return;
                        }
                    };

                    let ctx = (self.ctx_fn)(window.clone());
                    let window = window.clone();
                    let subscriptions = subscriptions.clone();
                    let executor = self.executor.clone();

                    // TODO: Remove spawn and queue instead?
                    TokioRuntime::spawn(async move {
                        todo!();
                        // let result = executor
                        //     .execute_batch(
                        //         ctx,
                        //         reqs,
                        //         &mut Some(TauriSubscriptionManager {
                        //             subscriptions,
                        //             window: window.clone(),
                        //         }),
                        //     )
                        //     .await;

                        // window
                        //     .emit(
                        //         "plugin:rspc:transport:resp",
                        //         serde_json::to_string(&result).unwrap(),
                        //     )
                        //     .map_err(|err| {
                        //         #[cfg(feature = "tracing")]
                        //         tracing::error!("failed to emit JSON-RPC response: {}", err);
                        //     })
                        //     .ok();
                    });
                }
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
    router: Arc<CompiledRouter<TCtx>>,
    ctx_fn: impl Fn(Window<tauri::Wry>) -> TCtx + Send + Sync + 'static,
) -> TauriPlugin<tauri::Wry>
where
    TCtx: Clone + Send + Sync + 'static,
{
    let manager = WindowManager::new::<_, _, TokioRuntime>(ctx_fn, router);
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
