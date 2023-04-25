use std::{
    borrow::Cow,
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
    jsonrpc::{NewOrOldInput, RequestId, RequestInner, ResponseInner},
    AsyncMap, ProcedureKind, RequestContext,
};

pub enum UnreachableSender {}
impl OwnedSender for UnreachableSender {
    type SendFut<'a> = Ready<()>;

    fn send(&mut self, _: jsonrpc::Response) -> Self::SendFut<'_> {
        // Fun fact: Cause this method takes `self` and `enum {}` can never be constructed, this function is impossible to run.
        unreachable!()
    }
}

pub enum SubscriptionUpgrade<'a, S: Sender<'a>> {
    Supported(S::OwnedSender, S::SubscriptionMap),
    Unsupported(S),
}

// TODO: Removing `Sync`?
pub trait Sender<'a>: Sized + Send + Sync {
    type SendFut: Future<Output = ()> + Send + Sync;
    type SubscriptionMap: AsyncMap<RequestId, oneshot::Sender<()>> + Sync + 'a;
    type OwnedSender: OwnedSender;

    fn subscription(self) -> SubscriptionUpgrade<'a, Self>;

    fn send(self, resp: jsonrpc::Response) -> Self::SendFut;
}

impl<'a> Sender<'a> for &'a mut Option<jsonrpc::Response> {
    type SendFut = Ready<()>;
    type SubscriptionMap = HashMap<RequestId, oneshot::Sender<()>>; // Unused
    type OwnedSender = UnreachableSender; // Unused

    fn subscription(self) -> SubscriptionUpgrade<'a, Self> {
        SubscriptionUpgrade::Unsupported(self)
    }

    fn send(self, resp: jsonrpc::Response) -> Self::SendFut {
        *self = Some(resp);
        ready(())
    }
}

pub struct SubscriptionSender<'a, S>(
    pub &'a mut futures_channel::mpsc::Sender<jsonrpc::Response>,
    pub S,
)
where
    S: AsyncMap<RequestId, oneshot::Sender<()>> + Sync;

impl<'a, S> Sender<'a> for SubscriptionSender<'a, S>
where
    S: AsyncMap<RequestId, oneshot::Sender<()>> + Sync + 'a,
{
    type SendFut = OwnedMpscSenderSendFut<'a>;
    type SubscriptionMap = S;
    type OwnedSender = OwnedMpscSender;

    fn subscription(self) -> SubscriptionUpgrade<'a, Self> {
        SubscriptionUpgrade::Supported(OwnedMpscSender(self.0.clone()), self.1)
    }

    fn send(self, resp: jsonrpc::Response) -> Self::SendFut {
        OwnedMpscSenderSendFut(self.0, Some(resp))
    }
}

pub trait OwnedSender: Send + Sync + 'static {
    type SendFut<'a>: Future<Output = ()> + Send + 'a;

    fn send(&mut self, resp: jsonrpc::Response) -> Self::SendFut<'_>;
}

pub struct OwnedMpscSender(futures_channel::mpsc::Sender<jsonrpc::Response>);

impl OwnedMpscSender {
    pub fn new(chan: futures_channel::mpsc::Sender<jsonrpc::Response>) -> Self {
        Self(chan)
    }
}

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
            Poll::Ready(Ok(_)) => {
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

// This is very intentionally not an `async fn`. It allows the `Send` bound to throw the error here instead of a cryptic `httpz` one.
#[allow(clippy::manual_async_fn)]
pub fn handle_json_rpc<'a, TCtx, TMeta>(
    ctx: TCtx,
    req: jsonrpc::Request,
    router: Cow<'a, Arc<Router<TCtx, TMeta>>>,
    sender: impl Sender<'a> + 'a,
) -> impl Future<Output = ()> + Send + 'a
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
{
    async move {
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
                    .queries
                    .store
                    .get(&path)
                    .ok_or_else(|| ExecError::OperationNotFound(path.clone()))
                {
                    Ok(op) => {
                        let mut stream = match op
                            .exec
                            .call(
                                ctx,
                                input.unwrap_or(Value::Null),
                                RequestContext {
                                    kind: ProcedureKind::Query,
                                    path,
                                },
                            )
                            .await
                        {
                            Ok(s) => s,
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
                                return;
                            }
                        };

                        // // TODO: Middleware could mess with this assumption so think about that.
                        // match stream.size_hint() {
                        //     (1, Some(1)) => {}
                        //     hint => {
                        //         #[cfg(debug_assertions)]
                        //         panic!("Internal rspc errror: invalid size hint {:?}", hint);
                        //         #[cfg(not(debug_assertions))]
                        //         return;
                        //     }
                        // }

                        match stream.next().await.unwrap() {
                            Ok(v) => {
                                sender
                                    .send(jsonrpc::Response {
                                        jsonrpc: "2.0",
                                        id: req.id,
                                        result: ResponseInner::Response(v),
                                    })
                                    .await;
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
                                return;
                            }
                        }
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
                }
            }
            RequestInner::Mutation { path, input } => {
                match router
                    .mutations
                    .store
                    .get(&path)
                    .ok_or_else(|| ExecError::OperationNotFound(path.clone()))
                {
                    Ok(op) => {
                        let mut stream = match op
                            .exec
                            .call(
                                ctx,
                                input.unwrap_or(Value::Null),
                                RequestContext {
                                    kind: ProcedureKind::Mutation,
                                    path,
                                },
                            )
                            .await
                        {
                            Ok(s) => s,
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
                                return;
                            }
                        };

                        // // TODO: Middleware could mess with this assumption so think about that.
                        // match stream.size_hint() {
                        //     (1, Some(1)) => {}
                        //     hint => {
                        //         #[cfg(debug_assertions)]
                        //         panic!("Internal rspc errror: invalid size hint {:?}", hint);
                        //         #[cfg(not(debug_assertions))]
                        //         return;
                        //     }
                        // }

                        match stream.next().await.unwrap() {
                            Ok(v) => {
                                sender
                                    .send(jsonrpc::Response {
                                        jsonrpc: "2.0",
                                        id: req.id,
                                        result: ResponseInner::Response(v),
                                    })
                                    .await;
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
                                return;
                            }
                        }
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
                }
            }
            RequestInner::Subscription { path, input } => match sender.subscription() {
                SubscriptionUpgrade::Supported(mut sender, mut subscriptions) => {
                    let (id, input) = match input {
                        NewOrOldInput::New(id, input) => (id, input),
                        NewOrOldInput::Old(input) => (req.id, input),
                    };

                    if matches!(id, RequestId::Null) {
                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: id.clone(),
                                result: ResponseInner::Error(
                                    ExecError::ErrSubscriptionWithNullId.into(),
                                ),
                            })
                            .await;
                    } else if subscriptions.contains_key(&id).await {
                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id: id.clone(),
                                result: ResponseInner::Error(
                                    ExecError::ErrSubscriptionDuplicateId.into(),
                                ),
                            })
                            .await;
                    }

                    if let Err(err) = router
                        .subscriptions
                        .store
                        .get(&path)
                        .ok_or_else(|| ExecError::OperationNotFound(path.clone()))
                    {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error executing operation: {:?}", err);

                        sender
                            .send(jsonrpc::Response {
                                jsonrpc: "2.0",
                                id,
                                result: ResponseInner::Error(err.into()),
                            })
                            .await;
                        return;
                    }

                    let router = to_owned(router);
                    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                    subscriptions.insert(id.clone(), shutdown_tx).await;
                    tokio::spawn(async move {
                        let op = router.subscriptions.store.get(&path).expect(
                            "Fatal rspc error: Rust's borrowing rules have been broken. An `&T` was modified.",
                        );

                        let mut stream = match op
                            .exec
                            .call(
                                ctx,
                                input.unwrap_or(Value::Null),
                                RequestContext {
                                    kind: ProcedureKind::Query,
                                    path,
                                },
                            )
                            .await
                        {
                            Ok(s) => s,
                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!("Error executing operation: {:?}", err);

                                sender
                                    .send(jsonrpc::Response {
                                        jsonrpc: "2.0",
                                        id: id,
                                        result: ResponseInner::Error(err.into()),
                                    })
                                    .await;
                                return;
                            }
                        };

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
                                            sender.send(jsonrpc::Response {
                                                jsonrpc: "2.0",
                                                id: id.clone(),
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
                SubscriptionUpgrade::Unsupported(sender) => {
                    sender
                        .send(jsonrpc::Response {
                            jsonrpc: "2.0",
                            id: req.id.clone(),
                            result: ResponseInner::Error(
                                ExecError::UnsupportedMethod("Subscription".to_string()).into(),
                            ),
                        })
                        .await;
                }
            },
            RequestInner::SubscriptionStop(input) => match sender.subscription() {
                SubscriptionUpgrade::Supported(_sender, mut subscriptions) => {
                    subscriptions
                        // We `unwrap_or` for backwards compatibility with the tRPC style client I had. // TODO: Remove this in the future
                        .remove(&input.map(|i| i.input).unwrap_or(req.id))
                        .await;
                }
                SubscriptionUpgrade::Unsupported(sender) => {
                    sender
                        .send(jsonrpc::Response {
                            jsonrpc: "2.0",
                            id: req.id.clone(),
                            result: ResponseInner::Error(
                                ExecError::UnsupportedMethod("Subscription".to_string()).into(),
                            ),
                        })
                        .await;
                }
            },
        };
    }
}

// TODO: Can this we removed?
fn to_owned<'a, T>(arc: Cow<'a, Arc<T>>) -> Arc<T> {
    match arc {
        Cow::Borrowed(arc) => arc.to_owned(),
        Cow::Owned(arc) => arc,
    }
}
