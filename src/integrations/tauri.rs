use std::{collections::HashMap, sync::Arc};

use futures::StreamExt;
use serde_json::Value;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::{mpsc, oneshot};

use crate::{
    internal::{
        jsonrpc::{self, RequestId, RequestInner, ResponseInner},
        ProcedureKind, RequestContext, ValueOrStream,
    },
    ExecError, Router,
};

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
            let (tx, mut rx) = mpsc::unbounded_channel::<jsonrpc::Request>();
            let (mut resp_tx, mut resp_rx) = mpsc::unbounded_channel::<jsonrpc::Response>();
            let mut subscriptions = HashMap::new();

            {
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    while let Some(req) = rx.recv().await {
                        let mut x = Sender::ResponseChannel((&mut resp_tx, &mut subscriptions));
                        handle_json_rpc(ctx_fn(), req, &router, &mut x).await;
                    }
                });
            }

            {
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    while let Some(event) = resp_rx.recv().await {
                        app_handle
                            .emit_all("plugin:rspc:transport:resp", event)
                            .unwrap();
                    }
                });
            }

            app_handle.listen_global("plugin:rspc:transport", move |event| {
                tx.send(serde_json::from_str(event.payload().unwrap()).unwrap())
                    .unwrap();
            });

            Ok(())
        })
        .build()
}

// TODO: Deduplicate this function with the httpz integration
pub enum Sender<'a> {
    Channel(
        (
            &'a mut mpsc::Sender<jsonrpc::Response>,
            &'a mut HashMap<RequestId, oneshot::Sender<()>>,
        ),
    ),
    ResponseChannel(
        (
            &'a mut mpsc::UnboundedSender<jsonrpc::Response>,
            &'a mut HashMap<RequestId, oneshot::Sender<()>>,
        ),
    ),
    Response(Option<jsonrpc::Response>),
}

impl<'a> Sender<'a> {
    pub async fn send(
        &mut self,
        resp: jsonrpc::Response,
    ) -> Result<(), mpsc::error::SendError<jsonrpc::Response>> {
        match self {
            Sender::Channel((tx, _)) => tx.send(resp).await?,
            Sender::ResponseChannel((tx, _)) => tx.send(resp)?,
            Sender::Response(o) => *o = Some(resp),
        }

        Ok(())
    }
}

pub async fn handle_json_rpc<TCtx, TMeta>(
    ctx: TCtx,
    req: jsonrpc::Request,
    router: &Arc<Router<TCtx, TMeta>>,
    tx: &mut Sender<'_>,
) where
    TCtx: 'static,
{
    if !req.jsonrpc.is_none() && req.jsonrpc.as_deref() != Some("2.0") {
        tx.send(jsonrpc::Response {
            jsonrpc: "2.0",
            id: req.id.clone(),
            result: ResponseInner::Error(ExecError::InvalidJsonRpcVersion.into()),
        })
        .await
        .unwrap();
    }

    let (path, input, procedures, sub_id) = match req.inner {
        RequestInner::Query { path, input } => (path, input, router.queries(), None),
        RequestInner::Mutation { path, input } => (path, input, router.mutations(), None),
        RequestInner::Subscription { path, input } => {
            (path, input.1, router.subscriptions(), Some(input.0))
        }
        RequestInner::SubscriptionStop { input } => {
            match tx {
                Sender::Channel((_, subscriptions)) => {
                    subscriptions.remove(&input);
                }
                Sender::ResponseChannel((_, subscriptions)) => {
                    subscriptions.remove(&input);
                }
                Sender::Response(_) => {}
            }

            return;
        }
    };

    let result = match procedures
        .get(&path)
        .ok_or_else(|| ExecError::OperationNotFound(path.clone()))
        .and_then(|v| {
            v.exec.call(
                ctx,
                input.unwrap_or(Value::Null),
                RequestContext {
                    kind: ProcedureKind::Query,
                    path,
                },
            )
        }) {
        Ok(op) => match op.into_value_or_stream().await {
            Ok(ValueOrStream::Value(v)) => ResponseInner::Response(v),
            Ok(ValueOrStream::Stream(mut stream)) => {
                let (tx, subscriptions) = match tx {
                    Sender::Channel((tx, subscriptions)) => (tx.clone(), subscriptions),
                    Sender::ResponseChannel((tx, subscriptions)) => todo!(),
                    Sender::Response(_) => {
                        todo!();
                    }
                };

                let id = sub_id.unwrap();
                if matches!(id, RequestId::Null) {
                    todo!();
                } else if subscriptions.contains_key(&id) {
                    todo!();
                }

                let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                subscriptions.insert(id.clone(), shutdown_tx);
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            biased; // Note: Order matters
                            _ = &mut shutdown_rx => {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("Removing subscription with id '{:?}'", id);
                                break;
                            }
                            v = stream.next() => {
                                match v {
                                    Some(v) => {
                                        tx.send(jsonrpc::Response {
                                            jsonrpc: "2.0",
                                            id: id.clone(),
                                            result: ResponseInner::Event(v.unwrap()),
                                        })
                                        .await
                                        .unwrap();
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });

                return;
            }
            Err(err) => ResponseInner::Error(err.into()),
        },
        Err(err) => ResponseInner::Error(err.into()),
    };

    tx.send(jsonrpc::Response {
        jsonrpc: "2.0",
        id: req.id,
        result,
    })
    .await
    .unwrap();
}
