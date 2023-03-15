use std::{
    collections::HashMap,
    future::{ready, Future, Ready},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::StreamExt;
use serde_json::Value;
use tokio::sync::oneshot;

use crate::{internal::jsonrpc, ExecError, Router};

use super::{
    jsonrpc::{RequestId, RequestInner, ResponseInner},
    AsyncMap, ProcedureKind, RequestContext, ValueOrStream,
};

// TODO: Deduplicate this function with the httpz integration

pub trait OwnedSender: Send + Sync + 'static {
    type SendFut<'a>: Future<Output = ()> + Send + 'a;

    fn send(&mut self, resp: jsonrpc::Response) -> Self::SendFut<'_>;
}

pub enum SubscriptionUpgrade<'s: 'm, 'm, S: Sender<'s, 'm>> {
    Supported(S::OwnedSender, &'m mut S::SubscriptionMap),
    Unsupported(S),
}

pub trait Sender<'s: 'm, 'm>: Sized + Send {
    type SendFut: Future<Output = ()> + Send + Sync;
    type SubscriptionMap: AsyncMap<RequestId, oneshot::Sender<()>> + Send + Sync;
    type OwnedSender: OwnedSender;

    fn subscription(self) -> SubscriptionUpgrade<'s, 'm, Self>;

    // We take `self` here so that you can only send a single response
    // Subscriptions require multiple responses but they get an [OwnedSender] to use for that.
    fn send(self, resp: jsonrpc::Response) -> Self::SendFut;
}

impl<'s: 'm, 'm> Sender<'s, 'm> for &'s mut Option<jsonrpc::Response> {
    type SendFut = Ready<()>;
    type SubscriptionMap = HashMap<RequestId, oneshot::Sender<()>>; // Unused
    type OwnedSender = UnreachableSender; // Unused

    fn subscription(self) -> SubscriptionUpgrade<'s, 'm, Self> {
        SubscriptionUpgrade::Unsupported(self)
    }

    fn send(mut self, resp: jsonrpc::Response) -> Self::SendFut {
        *self = Some(resp);
        ready(())
    }
}

pub enum UnreachableSender {}
impl OwnedSender for UnreachableSender {
    type SendFut<'a> = Ready<()>;

    fn send(&mut self, resp: jsonrpc::Response) -> Self::SendFut<'_> {
        // Fun fact: Cause this method takes `self` and `enum {}` can never be constructed, this function is impossible to run.
        unreachable!()
    }
}

// TODO: Generic on `S`
pub struct SubscriptionSender<'m> {
    pub tx: futures_channel::mpsc::Sender<jsonrpc::Response>,
    pub subscriptions: &'m mut HashMap<RequestId, oneshot::Sender<()>>,
}

impl<'s: 'm, 'm> Sender<'s, 'm> for &'s mut SubscriptionSender<'m> {
    type SendFut = OwnedMpscSenderSendFut<'s>;
    type SubscriptionMap = HashMap<RequestId, oneshot::Sender<()>>;
    type OwnedSender = OwnedMpscSender;

    fn subscription(self) -> SubscriptionUpgrade<'s, 'm, Self> {
        SubscriptionUpgrade::Supported(OwnedMpscSender(self.tx.clone()), self.subscriptions)
        // todo!();
    }

    fn send(self, resp: jsonrpc::Response) -> Self::SendFut {
        OwnedMpscSenderSendFut(&mut self.tx, Some(resp))
    }
}

pub struct OwnedMpscSender(futures_channel::mpsc::Sender<jsonrpc::Response>);

impl OwnedSender for OwnedMpscSender {
    type SendFut<'a> = OwnedMpscSenderSendFut<'a>;

    fn send(&mut self, resp: jsonrpc::Response) -> Self::SendFut<'_> {
        OwnedMpscSenderSendFut(&mut self.0, Some(resp))
    }
}

pub struct OwnedMpscSenderSendFut<'a>(
    &'a mut futures_channel::mpsc::Sender<jsonrpc::Response>,
    Option<jsonrpc::Response>,
);

impl<'a> Future for OwnedMpscSenderSendFut<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.0.poll_ready(cx) {
            Poll::Ready(Ok(permit)) => {
                this.0
                    .try_send(this.1.take().expect("Future polled after completion"))
                    .map_err(|_err| {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Failed to send response: {}", _err);
                    })
                    .ok();
                Poll::Ready(())
            }
            Poll::Ready(Err(_err)) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Failed to reserve capacity to send response: {}", _err);
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// TODO: Remove these or make them better
pub fn fuck_you<'s: 'm, 'm>(input: &'s mut Option<jsonrpc::Response>) -> impl Sender<'s, 'm> + 's {
    input
}

pub fn fuck_you2<'s: 'm, 'm>(input: &'s mut SubscriptionSender<'m>) -> impl Sender<'s, 'm> + 's {
    input
}

// TODO: This function is disgusting but the types make it hard to clean up
pub async fn handle_json_rpc<'s: 'm, 'm, TCtx, TMeta>(
    ctx: TCtx,
    req: jsonrpc::Request,
    router: &Arc<Router<TCtx, TMeta>>,
    sender: impl Sender<'s, 'm> + 's,
) where
    TCtx: 'static,
{
    if req.jsonrpc.is_some() && req.jsonrpc.as_deref() != Some("2.0") {
        sender
            .send(jsonrpc::Response {
                jsonrpc: "2.0",
                id: req.id.clone(),
                result: ResponseInner::Error(ExecError::InvalidJsonRpcVersion.into()),
            })
            .await;
        return;
    }

    match req.inner {
        RequestInner::Query { path, input } => {
            match router
                .queries()
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
                    Ok(ValueOrStream::Value(v)) => {
                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id,
                                result: ResponseInner::Response(v),
                            })
                            .await
                    }
                    Ok(ValueOrStream::Stream(stream)) => {
                        unreachable!();
                    }
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error executing operation: {:?}", err);

                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id,
                                result: ResponseInner::Error(err.into()),
                            })
                            .await;
                    }
                },
                Err(err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error executing operation: {:?}", err);

                    sender
                        .send(jsonrpc::Response {
                            jsonrpc: "2.0",
                            id: req.id,
                            result: ResponseInner::Error(err.into()),
                        })
                        .await;
                }
            }
        }
        RequestInner::Mutation { path, input } => {
            match router
                .mutations()
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
                    Ok(ValueOrStream::Value(v)) => {
                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id,
                                result: ResponseInner::Response(v),
                            })
                            .await
                    }
                    Ok(ValueOrStream::Stream(stream)) => {
                        unreachable!();
                    }
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error executing operation: {:?}", err);

                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id,
                                result: ResponseInner::Error(err.into()),
                            })
                            .await;
                    }
                },
                Err(err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error executing operation: {:?}", err);

                    sender
                        .send(jsonrpc::Response {
                            jsonrpc: "2.0",
                            id: req.id,
                            result: ResponseInner::Error(err.into()),
                        })
                        .await;
                }
            }
        }
        RequestInner::Subscription { path, input } => match sender.subscription() {
            SubscriptionUpgrade::Supported(mut sender, subscriptions) => {
                match router
                    .subscriptions()
                    .get(&path)
                    .ok_or_else(|| ExecError::OperationNotFound(path.clone()))
                    .and_then(|v| {
                        v.exec.call(
                            ctx,
                            input.1.unwrap_or(Value::Null),
                            RequestContext {
                                kind: ProcedureKind::Query,
                                path,
                            },
                        )
                    }) {
                    Ok(op) => match op.into_value_or_stream().await {
                        Ok(ValueOrStream::Value(v)) => {
                            unreachable!();
                        }
                        Ok(ValueOrStream::Stream(mut stream)) => {
                            if matches!(input.0, RequestId::Null) {
                                sender
                                    .send(jsonrpc::Response {
                                        jsonrpc: "2.0",
                                        id: req.id.clone(),
                                        result: ResponseInner::Error(
                                            ExecError::ErrSubscriptionWithNullId.into(),
                                        ),
                                    })
                                    .await;
                            } else if subscriptions.contains_key(&input.0).await {
                                sender
                                    .send(jsonrpc::Response {
                                        jsonrpc: "2.0",
                                        id: req.id.clone(),
                                        result: ResponseInner::Error(
                                            ExecError::ErrSubscriptionDuplicateId.into(),
                                        ),
                                    })
                                    .await;
                            }

                            let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                            subscriptions.insert(input.0.clone(), shutdown_tx).await;
                            tokio::spawn(async move {
                                loop {
                                    tokio::select! {
                                        biased; // Note: Order matters
                                        _ = &mut shutdown_rx => {
                                            #[cfg(feature = "tracing")]
                                            tracing::debug!("Removing subscription with id '{:?}'", input.0);
                                            break;
                                        }
                                        v = stream.next() => {
                                            match v {
                                                Some(Ok(v)) => {
                                                sender.send(jsonrpc::Response {
                                                        jsonrpc: "2.0",
                                                        id: input.0.clone(),
                                                        result: ResponseInner::Event(v),
                                                    })
                                                    .await;
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
                        Err(err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error executing operation: {:?}", err);

                            sender
                                .send(jsonrpc::Response {
                                    jsonrpc: "2.0",
                                    id: req.id,
                                    result: ResponseInner::Error(err.into()),
                                })
                                .await;
                        }
                    },
                    Err(err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error executing operation: {:?}", err);

                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: req.id,
                                result: ResponseInner::Error(err.into()),
                            })
                            .await;
                    }
                }
            }
            SubscriptionUpgrade::Unsupported(sender) => {
                todo!();
            }
        },

        // RequestInner::SubscriptionStop { input } => {
        //     match sender.subscriptions() {
        //         SubscriptionUpgrade::Supported(s, mut m) => {
        //             m.remove(&input).await;
        //         }
        //         SubscriptionUpgrade::Unsupported(sender) => {
        //             sender
        //                 .send(jsonrpc::Response {
        //                     jsonrpc: "2.0",
        //                     id: req.id.clone(),
        //                     result: ResponseInner::Error(
        //                         ExecError::UnsupportedMethod("Subscription".to_string()).into(),
        //                     ),
        //                 })
        //                 .await;
        //         }
        //     }
        //     return;
        // }
        _ => todo!(),
    };
}
