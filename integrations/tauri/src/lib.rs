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
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use rspc_core::{ProcedureError, Procedures};
use serde::{Deserialize, Serialize};
use serde_json::{value::Serializer, Value};
use tauri::{
    async_runtime::{spawn, JoinHandle},
    generate_handler,
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
        channel: tauri::ipc::Channel<Response>,
        req: Request,
    ) {
        match req {
            Request::Request { path, input } => {
                let ctx = (self.ctx_fn)(window);

                let id = channel.id();
                let send = move |resp: Option<Result<Value, ProcedureError<Serializer>>>| {
                    channel
                        .send(
                            resp.ok_or(Response::Done)
                                .and_then(|v| {
                                    v.map(|value| Response::Value { code: 200, value }).map_err(
                                        |err| Response::Value {
                                            code: err.code(),
                                            value: serde_json::to_value(err).unwrap(), // TODO: Error handling (can we throw it back into Tauri, else we are at an impasse)
                                        },
                                    )
                                })
                                .unwrap_or_else(|e| e),
                        )
                        .ok()
                };

                let Some(procedure) = self.procedures.get(&Cow::Borrowed(&*path)) else {
                    send(Some(Err(ProcedureError::NotFound)));
                    send(None);
                    return;
                };

                let mut stream =
                    procedure.exec_with_deserializer(ctx, input.unwrap_or(Value::Null));

                let this = self.clone();
                let handle = spawn(async move {
                    loop {
                        let value = stream.next(Serializer).await;
                        let is_finished = value.is_none();
                        send(value);

                        if is_finished {
                            break;
                        }
                    }

                    this.subscriptions().remove(&id);
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
    channel: tauri::ipc::Channel<Response>,
    req: Request,
) {
    state.0.clone().handle_rpc(window, channel, req);
}

pub fn plugin<R, TCtxFn, TCtx>(
    procedures: impl Into<Procedures<TCtx>>,
    ctx_fn: TCtxFn,
) -> TauriPlugin<R>
where
    R: tauri::Runtime + Send + Sync,
    TCtxFn: Fn(tauri::Window<R>) -> TCtx + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
{
    let procedures = procedures.into();

    Builder::new("rspc")
        .invoke_handler(generate_handler![handle_rpc])
        .setup(move |app_handle, _| {
            if app_handle.manage(State(Arc::new(RpcHandler {
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
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
enum Request {
    /// A request to execute a procedure.
    Request { path: String, input: Option<Value> },
    /// Abort a running task
    /// You must provide the ID of the Tauri channel provided when the task was started.
    Abort(u32),
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
enum Response {
    /// A value being returned from a procedure.
    /// Based on the code we can determine if it's an error or not.
    Value { code: u16, value: Value },
    /// A procedure has been completed.
    /// It's important you avoid calling `Request::Abort { id }` after this as it's up to Tauri what happens.
    Done,
}
