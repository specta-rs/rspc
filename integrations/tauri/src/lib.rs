//! [Tauri](https://tauri.app) integration for [rspc](https://rspc.dev).
//!
//! # Example
//!
//! ```rust
//! use rspc::Router;
//!
//! let router = Router::new();
//! let (procedures, _types) = router.build().unwrap();
//!
//! tauri::Builder::default()
//!     .plugin(tauri_plugin_rspc::init(procedures, |window| todo!()))
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```
//!
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use rspc_procedure::{ProcedureError, Procedures};
use serde::{de::Error, Deserialize, Serialize};
use serde_json::value::RawValue;
use tauri::{
    async_runtime::{spawn, JoinHandle},
    generate_handler,
    ipc::{Channel, InvokeResponseBody, IpcResponse},
    plugin::{Builder, TauriPlugin},
    Manager,
};

struct RpcHandler<R, TCtxFn, TCtx> {
    subscriptions: Mutex<HashMap<u32, JoinHandle<()>>>,
    ctx_fn: TCtxFn,
    procedures: Procedures<TCtx>,
    phantom: std::marker::PhantomData<fn() -> R>,
}

impl<R, TCtxFn, TCtx> RpcHandler<R, TCtxFn, TCtx>
where
    R: tauri::Runtime,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    fn subscriptions(&self) -> MutexGuard<HashMap<u32, JoinHandle<()>>> {
        self.subscriptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn handle_rpc_impl(
        self: Arc<Self>,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<IpcResultResponse>,
        req: Request,
    ) {
        match req {
            Request::Request { path, input } => {
                let id = channel.id();
                let ctx = (self.ctx_fn)(window);

                let Some(procedure) = self.procedures.get(&Cow::Borrowed(&*path)) else {
                    let err = ProcedureError::NotFound;
                    send(
                        &channel,
                        Response::Value {
                            code: match err {
                                ProcedureError::NotFound => 404,
                                ProcedureError::Deserialize(_) => 400,
                                ProcedureError::Downcast(_) => 400,
                                ProcedureError::Resolver(_) => 500, // This is a breaking change. It previously came from the user.
                                ProcedureError::Unwind(_) => 500,
                            },
                            value: &err,
                        },
                    );
                    send::<()>(&channel, Response::Done);
                    return;
                };

                let mut stream = match input {
                    Some(i) => procedure.exec_with_deserializer(ctx, i.as_ref()),
                    None => procedure.exec_with_deserializer(ctx, serde_json::Value::Null),
                };

                let this = self.clone();
                let handle = spawn(async move {
                    while let Some(value) = stream.next().await {
                        match value {
                            Ok(v) => send(
                                &channel,
                                Response::Value {
                                    code: 200,
                                    value: &v.as_serialize().unwrap(),
                                },
                            ),
                            Err(err) => send(
                                &channel,
                                Response::Value {
                                    code: match err {
                                        ProcedureError::NotFound => 404,
                                        ProcedureError::Deserialize(_) => 400,
                                        ProcedureError::Downcast(_) => 400,
                                        ProcedureError::Resolver(_) => 500, // This is a breaking change. It previously came from the user.
                                        ProcedureError::Unwind(_) => 500,
                                    },
                                    value: &err,
                                },
                            ),
                        }
                    }

                    this.subscriptions().remove(&id);
                    send::<()>(&channel, Response::Done);
                });

                // if the client uses an existing ID, we will assume the previous subscription is no longer required
                if let Some(old) = self.subscriptions().insert(id, handle) {
                    old.abort();
                }
            }
            Request::Abort(id) => {
                if let Some(h) = self.subscriptions().remove(&id) {
                    h.abort();
                }
            }
        }
    }
}

trait HandleRpc<R: tauri::Runtime>: Send + Sync {
    fn handle_rpc(
        self: Arc<Self>,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<IpcResultResponse>,
        req: Request,
    );
}

impl<R, TCtxFn, TCtx> HandleRpc<R> for RpcHandler<R, TCtxFn, TCtx>
where
    R: tauri::Runtime,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    fn handle_rpc(
        self: Arc<Self>,
        window: tauri::Window<R>,
        channel: tauri::ipc::Channel<IpcResultResponse>,
        req: Request,
    ) {
        Self::handle_rpc_impl(self, window, channel, req);
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
    channel: tauri::ipc::Channel<IpcResultResponse>,
    req: Request,
) {
    state.0.clone().handle_rpc(window, channel, req);
}

pub fn init<R, TCtxFn, TCtx>(
    procedures: impl Into<Procedures<TCtx>>,
    ctx_fn: TCtxFn,
) -> TauriPlugin<R>
where
    R: tauri::Runtime,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
{
    let procedures = procedures.into();

    Builder::new("rspc")
        .invoke_handler(generate_handler![handle_rpc])
        .setup(move |app_handle, _| {
            if !app_handle.manage(State(Arc::new(RpcHandler {
                subscriptions: Default::default(),
                ctx_fn,
                procedures,
                phantom: Default::default(),
            }))) {
                panic!("Attempted to mount `rspc_tauri::plugin` multiple times. Please ensure you only mount it once!");
            }

            Ok(())
        })
        .build()
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Request {
    /// A request to execute a procedure.
    Request {
        path: String,
        // #[serde(borrow)]
        input: Option<Box<RawValue>>,
    },
    /// Abort a running task
    /// You must provide the ID of the Tauri channel provided when the task was started.
    Abort(u32),
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response<'a, T: Serialize> {
    Value { code: u16, value: &'a T },
    Done,
}

fn send<'a, T: Serialize>(channel: &Channel<IpcResultResponse>, value: Response<'a, T>) {
    channel
        .send(IpcResultResponse(
            serde_json::to_string(&value)
                .map(|value| InvokeResponseBody::Json(value))
                .map_err(|err| err.to_string()),
        ))
        .ok();
}

#[derive(Clone)]
struct IpcResultResponse(Result<InvokeResponseBody, String>);

impl IpcResponse for IpcResultResponse {
    fn body(self) -> tauri::Result<InvokeResponseBody> {
        self.0.map_err(|err| serde_json::Error::custom(err).into())
    }
}
