use nanoid::nanoid;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::StreamExt;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{Error, ErrorCode, OperationKind, Request, Response, Router, StreamOrValue};

impl Request {
    pub async fn handle<TCtx, TMeta>(
        self,
        ctx: TCtx,
        router: &Arc<Router<TCtx, TMeta>>,
        client_ctx: &ClientContext,
        event_sender: Option<&UnboundedSender<Response>>,
    ) -> Response
    where
        TCtx: Send + 'static,
        TMeta: Send + Sync + 'static,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!("Handling request: {:?}", self);
        match self.operation {
            OperationKind::Query | OperationKind::Mutation => {
                match router
                    .exec(ctx, self.operation.clone(), self.key.clone())
                    .await
                {
                    Ok(result) => match result {
                        StreamOrValue::Stream(_) => unreachable!(),
                        StreamOrValue::Value(v) => Response::Response {
                            id: self.id,
                            result: v,
                        },
                    },
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!(
                            "error executing operation {:?} with id '{}': {:?}",
                            self.operation,
                            self.id.clone().unwrap_or("".into()),
                            err
                        );
                        err.into_rspc_err().into_response(self.id)
                    }
                }
            }
            OperationKind::SubscriptionAdd => {
                // TODO: Scope websocket to the client IP/Port or something so that another client can't unsubscribe them.
                let subscription_id = nanoid!();
                let mut shutdown_rx = client_ctx.register(
                    self.operation.to_string(),
                    self.key.0.clone(),
                    subscription_id.clone(),
                );

                match router
                    .exec(ctx, self.operation.clone(), self.key.clone())
                    .await
                {
                    Ok(result) => match result {
                        StreamOrValue::Stream(mut stream) => match event_sender {
                            Some(event_sender) => {
                                let event_sender = event_sender.clone();
                                let key = self.key.0.clone();
                                {
                                    let subscription_id = subscription_id.clone();
                                    tokio::spawn(async move {
                                        loop {
                                            tokio::select! {
                                                msg = stream.next() => {
                                                    if let Some(msg) = msg {
                                                        let resp = match msg {
                                                            Ok(msg) => Response::Event{
                                                                id: subscription_id.clone(),
                                                                key: key.clone(),
                                                                result: msg,
                                                            },
                                                            Err(err) => {
                                                                #[cfg(feature = "tracing")]
                                                                tracing::error!(
                                                                    "error executing operation {:?} with id '{}': {:?}",
                                                                    self.operation,
                                                                    self.id.clone().unwrap_or("".into()),
                                                                    err
                                                                );

                                                                err.into_rspc_err().into_response(Some(subscription_id.clone()))
                                                            },
                                                        };


                                                        match event_sender.send(resp) {
                                                            Ok(_) => {},
                                                            Err(_) => {
                                                                #[cfg(feature = "tracing")]
                                                                tracing::warn!("subscription event was dropped, the server may be overloaded!");

                                                                println!("rspc: subscription event was dropped, the server may be overloaded!");
                                                            }
                                                        }
                                                    } else {
                                                        break;
                                                    }
                                                }
                                                _ = &mut shutdown_rx => {
                                                    break;
                                                }
                                            }
                                        }
                                    });
                                }
                                Response::Response {
                                    id: self.id,
                                    result: Value::String(subscription_id),
                                }
                            }
                            None => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(
                                    "error cannot add subscription without event sender"
                                );
                                Error {
                                    code: ErrorCode::InternalServerError,
                                    message: "Can't add subscription without event sender".into(),
                                    cause: None,
                                }
                                .into_response(self.id)
                            }
                        },
                        StreamOrValue::Value(_) => unreachable!(),
                    },
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!(
                            "error executing operation {:?} with id '{}': {:?}",
                            self.operation,
                            self.id.clone().unwrap_or("".into()),
                            err
                        );
                        err.into_rspc_err().into_response(self.id)
                    }
                }
            }
            OperationKind::SubscriptionRemove => {
                // TODO: This isn't secure, someone could remove another client's subscription
                let _ = client_ctx.unregister(
                    self.operation.to_string(),
                    self.key.0.clone(),
                    self.key.0.clone(), // Subscription ID
                ); // TODO: Handle result

                Response::None
            }
        }
    }
}

pub struct ClientContext {
    pub subscriptions: Mutex<
        HashMap<
            (
                /* operation key */ String,
                /* method */ String,
                /* id */ String,
            ),
            oneshot::Sender<()>,
        >,
    >,
}

impl ClientContext {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            subscriptions: Mutex::new(HashMap::new()),
        })
    }

    pub fn register(&self, operation: String, method: String, id: String) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.subscriptions
            .lock()
            .unwrap()
            .insert((operation, method, id), tx);
        rx
    }

    pub fn unregister(&self, operation: String, method: String, id: String) -> Result<(), ()> {
        self.subscriptions
            .lock()
            .unwrap()
            .remove(&(operation, method, id))
            .ok_or(())?
            .send(())
            .map(|_| ())
    }
}
