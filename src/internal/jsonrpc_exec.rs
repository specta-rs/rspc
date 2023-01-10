use std::{collections::HashMap, sync::Arc};

use futures::StreamExt;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

use crate::{internal::jsonrpc, ExecError, Router};

use super::{
    jsonrpc::{RequestId, RequestInner, ResponseInner},
    LayerReturn, ProcedureKind, RequestContext,
};

// TODO: Deduplicate this function with the httpz integration

pub enum SubscriptionMap<'a> {
    Ref(&'a mut HashMap<RequestId, oneshot::Sender<()>>),
    Mutex(&'a Mutex<HashMap<RequestId, oneshot::Sender<()>>>),
    None,
}

impl<'a> SubscriptionMap<'a> {
    pub async fn has_subscription(&self, id: &RequestId) -> bool {
        match self {
            SubscriptionMap::Ref(map) => map.contains_key(id),
            SubscriptionMap::Mutex(map) => {
                let map = map.lock().await;
                map.contains_key(id)
            }
            SubscriptionMap::None => unreachable!(),
        }
    }

    pub async fn insert(&mut self, id: RequestId, tx: oneshot::Sender<()>) {
        match self {
            SubscriptionMap::Ref(map) => {
                map.insert(id, tx);
            }
            SubscriptionMap::Mutex(map) => {
                let mut map = map.lock().await;
                map.insert(id, tx);
            }
            SubscriptionMap::None => unreachable!(),
        }
    }

    pub async fn remove(&mut self, id: &RequestId) {
        match self {
            SubscriptionMap::Ref(map) => {
                map.remove(id);
            }
            SubscriptionMap::Mutex(map) => {
                let mut map = map.lock().await;
                map.remove(id);
            }
            SubscriptionMap::None => unreachable!(),
        }
    }
}
pub enum Sender<'a> {
    Channel(&'a mut mpsc::Sender<jsonrpc::Response>),
    ResponseChannel(&'a mut mpsc::UnboundedSender<jsonrpc::Response>),
    Broadcast(&'a broadcast::Sender<jsonrpc::Response>),
    Response(Option<jsonrpc::Response>),
    // We don't use this internally but Spacedrive uses it for the React Native bridge.
    ResponseAndChannel(
        Option<jsonrpc::Response>,
        &'a mut mpsc::UnboundedSender<jsonrpc::Response>,
    ),
}

pub enum Sender2 {
    Channel(mpsc::Sender<jsonrpc::Response>),
    ResponseChannel(mpsc::UnboundedSender<jsonrpc::Response>),
    Broadcast(broadcast::Sender<jsonrpc::Response>),
}

impl Sender2 {
    pub async fn send(
        &mut self,
        resp: jsonrpc::Response,
    ) -> Result<(), mpsc::error::SendError<jsonrpc::Response>> {
        match self {
            Self::Channel(tx) => tx.send(resp).await?,
            Self::ResponseChannel(tx) => tx.send(resp)?,
            Self::Broadcast(tx) => {
                let _ = tx.send(resp).map_err(|_err| {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Failed to send response: {}", _err);
                });
            }
        }

        Ok(())
    }
}

impl<'a> Sender<'a> {
    pub async fn send(
        &mut self,
        resp: jsonrpc::Response,
    ) -> Result<(), mpsc::error::SendError<jsonrpc::Response>> {
        match self {
            Self::Channel(tx) => tx.send(resp).await?,
            Self::ResponseChannel(tx) => tx.send(resp)?,
            Self::Broadcast(tx) => {
                let _ = tx.send(resp).map_err(|_err| {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Failed to send response: {}", _err);
                });
            }
            Self::Response(r) => {
                *r = Some(resp);
            }
            Self::ResponseAndChannel(r, _) => {
                *r = Some(resp);
            }
        }

        Ok(())
    }

    pub fn sender2(&mut self) -> Sender2 {
        match self {
            Self::Channel(tx) => Sender2::Channel(tx.clone()),
            Self::ResponseChannel(tx) => Sender2::ResponseChannel(tx.clone()),
            Self::Broadcast(tx) => Sender2::Broadcast(tx.clone()),
            Self::Response(_) => unreachable!(),
            Self::ResponseAndChannel(_, tx) => Sender2::ResponseChannel(tx.clone()),
        }
    }
}

pub async fn handle_json_rpc<TCtx>(
    ctx: TCtx,
    req: jsonrpc::Request,
    router: &Arc<Router<TCtx>>,
    sender: &mut Sender<'_>,
    subscriptions: &mut SubscriptionMap<'_>,
) where
    TCtx: 'static,
{
    if req.jsonrpc.is_some() && req.jsonrpc.as_deref() != Some("2.0") {
        let _ = sender
            .send(jsonrpc::Response {
                jsonrpc: "2.0",
                id: req.id.clone(),
                result: ResponseInner::Error(ExecError::InvalidJsonRpcVersion.into()),
            })
            .await
            .map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("Failed to send response: {}", _err);
            });
    }

    let (path, input, procedures, sub_id) = match req.inner {
        RequestInner::Query { path, input } => (path, input, router.queries(), None),
        RequestInner::Mutation { path, input } => (path, input, router.mutations(), None),
        RequestInner::Subscription { path, input } => {
            (path, input, router.subscriptions(), Some(req.id.clone()))
        }
        RequestInner::SubscriptionStop => {
            subscriptions.remove(&req.id).await;
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
        Ok(op) => match op.into_layer_return().await {
            Ok(LayerReturn::Request(v)) => ResponseInner::Response(v),
            Ok(LayerReturn::Stream(mut stream)) => {
                if matches!(sender, Sender::Response(_))
                    || matches!(subscriptions, SubscriptionMap::None)
                {
                    let _ = sender
                        .send(jsonrpc::Response {
                            jsonrpc: "2.0",
                            id: req.id.clone(),
                            result: ResponseInner::Error(
                                ExecError::UnsupportedMethod("Subscription".to_string()).into(),
                            ),
                        })
                        .await
                        .map_err(|_err| {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Failed to send response: {}", _err);
                        });
                }

                if let Some(id) = sub_id {
                    if matches!(id, RequestId::Null) {
                        let _ = sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id.clone(),
                                result: ResponseInner::Error(
                                    ExecError::ErrSubscriptionWithNullId.into(),
                                ),
                            })
                            .await
                            .map_err(|_err| {
                                #[cfg(feature = "tracing")]
                                tracing::error!("Failed to send response: {}", _err);
                            });
                    } else if subscriptions.has_subscription(&id).await {
                        let _ = sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id.clone(),
                                result: ResponseInner::Error(
                                    ExecError::ErrSubscriptionDuplicateId.into(),
                                ),
                            })
                            .await
                            .map_err(|_err| {
                                #[cfg(feature = "tracing")]
                                tracing::error!("Failed to send response: {}", _err);
                            });
                    }

                    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                    subscriptions.insert(id.clone(), shutdown_tx).await;
                    let mut sender2 = sender.sender2();
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
                                        Some(Ok(v)) => {
                                            let _ = sender2.send(jsonrpc::Response {
                                                jsonrpc: "2.0",
                                                id: id.clone(),
                                                result: ResponseInner::Event(v),
                                            })
                                            .await
                                            .map_err(|_err| {
                                                #[cfg(feature = "tracing")]
                                                tracing::error!("Failed to send response: {:?}", _err);
                                            });
                                        }
                                        Some(Err(_err)) => {
                                           #[cfg(feature = "tracing")]
                                            tracing::error!("Subscription error: {:?}", _err);
                                        }
                                        None => {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    });
                }

                return;
            }
            Err(err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Error executing operation: {:?}", err);

                ResponseInner::Error(err.into())
            }
        },
        Err(err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing operation: {:?}", err);
            ResponseInner::Error(err.into())
        }
    };

    let _ = sender
        .send(jsonrpc::Response {
            jsonrpc: "2.0",
            id: req.id,
            result,
        })
        .await
        .map_err(|_err| {
            #[cfg(feature = "tracing")]
            tracing::error!("Failed to send response: {:?}", _err);
        });
}
