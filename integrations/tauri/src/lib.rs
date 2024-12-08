//! rspc-tauri: Tauri integration for [rspc](https://rspc.dev).
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    task::Poll,
};

use futures::{pin_mut, stream, FutureExt, StreamExt};
use rspc_core::{ProcedureError, Procedures};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{
    async_runtime::spawn,
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager,
};
use tokio::sync::oneshot;

struct RpcHandler<R, TCtxFn, TCtx> {
    subscriptions: Mutex<HashMap<u32, oneshot::Sender<()>>>,
    ctx_fn: TCtxFn,
    procedures: Procedures<TCtx>,
    phantom: std::marker::PhantomData<R>,
}

impl<R, TCtxFn, TCtx> RpcHandler<R, TCtxFn, TCtx>
where
    R: tauri::Runtime,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    fn new(procedures: Procedures<TCtx>, ctx_fn: TCtxFn) -> Self {
        Self {
            subscriptions: Default::default(),
            ctx_fn,
            procedures,
            phantom: Default::default(),
        }
    }

    fn subscriptions(&self) -> MutexGuard<HashMap<u32, oneshot::Sender<()>>> {
        self.subscriptions.lock().unwrap()
    }

    async fn handle_rpc_impl(
        self: Arc<Self>,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<Response>,
        req: Request,
    ) {
        let (path, input, sub_id) = match req {
            Request::Query { path, input } | Request::Mutation { path, input } => {
                (path, input, None)
            }
            Request::Subscription { path, input, id } => (path, input, Some(id)),
            Request::SubscriptionStop { id } => {
                self.subscriptions().remove(&id);
                return;
            }
        };

        let ctx = (self.ctx_fn)(window);

        let resp = match self.procedures.get(&Cow::Borrowed(&*path)) {
            Some(procedure) => {
                let mut stream =
                    procedure.exec_with_deserializer(ctx, input.unwrap_or(Value::Null));

                // It's really important this is before getting the first value
                // Size hints can change after the first value is polled based on implementation.
                let is_value = stream.size_hint() == (1, Some(1));

                let first_value = stream.next(serde_json::value::Serializer).await;

                if (is_value || stream.size_hint() == (0, Some(0))) && first_value.is_some() {
                    first_value
                        .expect("checked at if above")
                        .map(Response::Response)
                        .unwrap_or_else(|err| {
                            // #[cfg(feature = "tracing")]
                            // tracing::error!("Error executing operation: {:?}", err);

                            Response::Error(match err {
                                ProcedureError::Deserialize(_) => Error {
                                    code: 400,
                                    message: "error deserializing procedure arguments".to_string(),
                                    data: None,
                                },
                                ProcedureError::Downcast(_) => Error {
                                    code: 400,
                                    message: "error downcasting procedure arguments".to_string(),
                                    data: None,
                                },
                                ProcedureError::Serializer(_) => Error {
                                    code: 500,
                                    message: "error serializing procedure result".to_string(),
                                    data: None,
                                },
                                ProcedureError::Resolver(resolver_error) => {
                                    let legacy_error = resolver_error
                                        .error()
                                        .and_then(|v| {
                                            v.downcast_ref::<rspc_core::LegacyErrorInterop>()
                                        })
                                        .cloned();

                                    Error {
                                        code: resolver_error.status() as i32,
                                        message: legacy_error
                                            .map(|v| v.0.clone())
                                            // This probally isn't a great format but we are assuming your gonna use the new router with a new executor for typesafe errors.
                                            .unwrap_or_else(|| resolver_error.to_string()),
                                        data: None,
                                    }
                                }
                            })
                        })
                } else {
                    let Some(id) = sub_id else {
                        return;
                    };

                    if self.subscriptions().contains_key(&id) {
                        Response::Error(Error {
                            code: 400,
                            message: "error creating subscription with duplicate id".into(),
                            data: None,
                        })
                    } else {
                        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                        self.subscriptions().insert(id.clone(), shutdown_tx);

                        let channel = channel.clone();
                        tokio::spawn(async move {
                            let mut first_value = Some(first_value);

                            let mut stream = stream::poll_fn(|cx| {
                                if let Some(first_value) = first_value.take() {
                                    return Poll::Ready(Some(first_value));
                                }

                                if let Poll::Ready(_) = shutdown_rx.poll_unpin(cx) {
                                    return Poll::Ready(None);
                                }

                                let stream_fut = stream.next(serde_json::value::Serializer);
                                pin_mut!(stream_fut);

                                stream_fut.poll_unpin(cx).map(|v| v.map(Some))
                            });

                            while let Some(event) = stream.next().await {
                                match event {
                                    Some(Ok(v)) => {
                                        channel.send(Response::Event(v)).ok();
                                    }
                                    Some(Err(_err)) => {
                                        // #[cfg(feature = "tracing")]
                                        //  tracing::error!("Subscription error: {:?}", _err);
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            }
                        });

                        return;
                    }
                }
            }
            None => {
                // #[cfg(feature = "tracing")]
                // tracing::error!("Error executing operation: the requested operation '{path}' is not supported by this server");
                Response::Error(Error {
                    code: 404,
                    message: "the requested operation is not supported by this server".to_string(),
                    data: None,
                })
            }
        };

        channel.send(resp).ok();
    }
}

trait HandleRpc<R: tauri::Runtime>: Send + Sync {
    fn handle_rpc(
        self: Arc<Self>,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<Response>,
        req: Request,
    );
}

impl<R, TCtxFn, TCtx> HandleRpc<R> for RpcHandler<R, TCtxFn, TCtx>
where
    R: tauri::Runtime + Send + Sync,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    fn handle_rpc(
        self: Arc<Self>,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<Response>,
        req: Request,
    ) {
        spawn(Self::handle_rpc_impl(self, window, channel, req));
    }
}

// Tauri commands can't be generic except for their runtime,
// so we need to store + access the handler behind a trait.
// This way handle_rpc_impl has full access to the generics it was instantiated with,
// while State can be stored a) as a singleton (enforced by the type system!) and b) as type erased Tauri state
struct State<R>(Arc<dyn HandleRpc<R>>);

#[tauri::command]
fn handle_rpc<R: tauri::Runtime>(
    state: tauri::State<'_, State<R>>,
    window: tauri::Window<R>,
    channel: tauri::ipc::Channel<Response>,
    req: Request,
) {
    state.0.clone().handle_rpc(window, channel, req);
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
            app_handle.manage(State(Arc::new(RpcHandler::new(routes, ctx_fn))));

            Ok(())
        })
        .build()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum Request {
    Query {
        path: String,
        input: Option<Value>,
    },
    Mutation {
        path: String,
        input: Option<Value>,
    },
    Subscription {
        path: String,
        id: u32,
        input: Option<Value>,
    },
    SubscriptionStop {
        id: u32,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum Response {
    Event(Value),
    Response(Value),
    Error(Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}
