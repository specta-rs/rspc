use std::{collections::HashMap, sync::Arc};

use futures::{Stream, StreamExt};
use rspc::ExecError;
use rspc_core::ProcedureError;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

use crate::{jsonrpc, v2::Routes};

use super::jsonrpc::{RequestId, RequestInner, ResponseInner};

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
        }

        Ok(())
    }

    pub fn sender2(&mut self) -> Sender2 {
        match self {
            Self::Channel(tx) => Sender2::Channel(tx.clone()),
            Self::ResponseChannel(tx) => Sender2::ResponseChannel(tx.clone()),
            Self::Broadcast(tx) => Sender2::Broadcast(tx.clone()),
            Self::Response(_) => unreachable!(),
        }
    }
}

pub async fn handle_json_rpc<TCtx>(
    ctx: TCtx,
    req: jsonrpc::Request,
    routes: &Routes<TCtx>,
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
                result: ResponseInner::Error(jsonrpc::JsonRPCError {
                    code: 400,
                    message: "invalid JSON-RPC version".into(),
                    data: None,
                }),
            })
            .await
            .map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("Failed to send response: {}", _err);
            });
    }

    let (path, input, sub_id) = match req.inner {
        RequestInner::Query { path, input } => (path, input, None),
        RequestInner::Mutation { path, input } => (path, input, None),
        RequestInner::Subscription { path, input } => (path, input.1, Some(input.0)),
        RequestInner::SubscriptionStop { input } => {
            subscriptions.remove(&input).await;
            return;
        }
    };

    let result = match routes.get(&path) {
        Some(procedure) => {
            let mut stream = procedure.exec_with_deserializer(ctx, input.unwrap_or(Value::Null));

            // It's really important this is before getting the first value
            // Size hints can change after the first value is polled based on implementation.
            let is_value = stream.size_hint() == (1, Some(1));

            let first_value = stream.next(serde_json::value::Serializer).await;

            if (is_value || stream.size_hint() == (0, Some(0))) && first_value.is_some() {
                first_value
                    .expect("checked at if above")
                    .map(ResponseInner::Response)
                    .unwrap_or_else(|err| {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error executing operation: {:?}", err);

                        ResponseInner::Error(match err {
                            ProcedureError::Deserialize(_) => jsonrpc::JsonRPCError {
                                code: 400,
                                message: "error deserializing procedure arguments".to_string(),
                                data: None,
                            },
                            ProcedureError::Downcast(_) => jsonrpc::JsonRPCError {
                                code: 400,
                                message: "error downcasting procedure arguments".to_string(),
                                data: None,
                            },
                            ProcedureError::Serializer(_) => jsonrpc::JsonRPCError {
                                code: 500,
                                message: "error serializing procedure result".to_string(),
                                data: None,
                            },
                            ProcedureError::Resolver(resolver_error) => {
                                let legacy_error = resolver_error
                                    .error()
                                    .and_then(|v| v.downcast_ref::<rspc_core::LegacyErrorInterop>())
                                    .cloned();

                                jsonrpc::JsonRPCError {
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
                if matches!(sender, Sender::Response(_))
                    || matches!(subscriptions, SubscriptionMap::None)
                {
                    let _ = sender
                        .send(jsonrpc::Response {
                            jsonrpc: "2.0",
                            id: req.id.clone(),
                            result: ResponseInner::Error(jsonrpc::JsonRPCError {
                                code: 400,
                                message: "unsupported metho".into(),
                                data: None,
                            }),
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
                                result: ResponseInner::Error(jsonrpc::JsonRPCError {
                                    code: 400,
                                    message: "error creating subscription with null request id"
                                        .into(),
                                    data: None,
                                }),
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
                                result: ResponseInner::Error(jsonrpc::JsonRPCError {
                                    code: 400,
                                    message: "error creating subscription with duplicate id".into(),
                                    data: None,
                                }),
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
                        match first_value {
                            Some(Ok(v)) => {
                                let _ = sender2
                                    .send(jsonrpc::Response {
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
                            None => return,
                        }

                        loop {
                            tokio::select! {
                                biased; // Note: Order matters
                                _ = &mut shutdown_rx => {
                                    #[cfg(feature = "tracing")]
                                    tracing::debug!("Removing subscription with id '{:?}'", id);
                                    break;
                                }
                                v = stream.next(serde_json::value::Serializer) => {
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
        }
        None => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing operation: the requested operation '{path}' is not supported by this server");
            ResponseInner::Error(jsonrpc::JsonRPCError {
                code: 404,
                message: "the requested operation is not supported by this server".to_string(),
                data: None,
            })
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
