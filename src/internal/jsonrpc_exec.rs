use std::{collections::HashMap, future::Future, marker::PhantomData, sync::Arc};

use futures::StreamExt;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

use crate::{internal::jsonrpc, ExecError, Router};

use super::{
    jsonrpc::{RequestId, RequestInner, ResponseInner},
    LayerReturn, ProcedureKind, RequestContext,
};

// TODO: Use this across whole of rspc and move outta this file
pub trait Fut<'a, TOut = ()>: Future<Output = TOut> + Send + 'a {}
impl<'a, TOut, TFut: Future<Output = TOut> + Send + 'a> Fut<'a, TOut> for TFut {}

pub trait SenderFn<'a>: FnMut(jsonrpc::Response) -> Self::Fut + Send + Sync + 'a {
    type Fut: Fut<'a>;
}
impl<'a, TFut: Fut<'a>, TFunc: FnMut(jsonrpc::Response) -> TFut + Send + Sync + 'a> SenderFn<'a>
    for TFunc
{
    type Fut = TFut;
}

pub trait Sender<'a>: 'a {
    type SendFut: Fut<'a>;
    // type SubscriptionInsertFut: Fut<'a>;
    // type SubscriptionHasFut: Fut<'a, bool>;
    // type SubscriptionRemoveFut: Fut<'a>;

    // fn supports_subscriptions(&self) -> bool;

    // fn insert_subscription(&mut self) -> Self::SubscriptionInsertFut;

    // fn has_subscription(&mut self) -> Self::SubscriptionHasFut;

    // fn remove_subscription(&mut self) -> Self::SubscriptionRemoveFut;

    fn send(&mut self, resp: jsonrpc::Response) -> Self::SendFut;
}

pub struct OneshotSender<'a, TFunc>
where
    TFunc: SenderFn<'a>,
{
    func: TFunc,
    phantom: PhantomData<&'a ()>,
}

impl<'a, TFunc> Sender<'a> for OneshotSender<'a, TFunc>
where
    TFunc: SenderFn<'a>,
{
    type SendFut = TFunc::Fut;
    // type SubscriptionInsertFut: Fut<'a>;
    // type SubscriptionHasFut: Fut<'a, bool>;
    // type SubscriptionRemoveFut: Fut<'a>;

    // fn supports_subscriptions(&self) -> bool;

    // fn insert_subscription(&mut self) -> Self::SubscriptionInsertFut;

    // fn has_subscription(&mut self) -> Self::SubscriptionHasFut;

    // fn remove_subscription(&mut self) -> Self::SubscriptionRemoveFut;

    fn send(&mut self, resp: jsonrpc::Response) -> Self::SendFut {
        (self.func)(resp)
    }
}

pub fn sender_fn<'a>(sender: impl SenderFn<'a>) -> impl Sender<'a> {
    OneshotSender {
        func: sender,
        phantom: PhantomData,
    }
}

pub async fn handle_json_rpc<'a, 'b, TCtx>(
    ctx: TCtx,
    req: jsonrpc::Request,
    router: &'b Arc<Router<TCtx>>,
    mut sender: impl Sender<'a>,
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
    }

    let (path, input, procedures, sub_id) = match req.inner {
        RequestInner::Query { path, input } => (path, input, router.queries(), None),
        RequestInner::Mutation { path, input } => (path, input, router.mutations(), None),
        RequestInner::Subscription { path, input } => {
            todo!();

            (path, input, router.subscriptions(), Some(req.id.clone())) // TODO: Avoid clone
        }
        RequestInner::SubscriptionStop => {
            // subscriptions.remove(&req.id).await;
            todo!();
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
                todo!();
                // if matches!(sender, Sender::Response(_))
                //     || matches!(subscriptions, SubscriptionMap::None)
                // {
                //     sender
                //         .send(jsonrpc::Response {
                //             jsonrpc: "2.0",
                //             id: req.id.clone(),
                //             result: ResponseInner::Error(
                //                 ExecError::UnsupportedMethod("Subscription".to_string()).into(),
                //             ),
                //         })
                //         .await;
                // }

                // if let Some(id) = sub_id {
                //     if matches!(id, RequestId::Null) {
                //         sender
                //             .send(jsonrpc::Response {
                //                 jsonrpc: "2.0",
                //                 id: req.id.clone(),
                //                 result: ResponseInner::Error(
                //                     ExecError::ErrSubscriptionWithNullId.into(),
                //                 ),
                //             })
                //             .await;
                //     } else if subscriptions.has_subscription(&id).await {
                //         sender
                //             .send(jsonrpc::Response {
                //                 jsonrpc: "2.0",
                //                 id: req.id.clone(),
                //                 result: ResponseInner::Error(
                //                     ExecError::ErrSubscriptionDuplicateId.into(),
                //                 ),
                //             })
                //             .await;
                //     }

                //     let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                //     subscriptions.insert(id.clone(), shutdown_tx).await;
                //     let mut sender2 = sender.sender2();
                //     tokio::spawn(async move {
                //         loop {
                //             tokio::select! {
                //                 biased; // Note: Order matters
                //                 _ = &mut shutdown_rx => {
                //                     #[cfg(feature = "tracing")]
                //                     tracing::debug!("Removing subscription with id '{:?}'", id);
                //                     break;
                //                 }
                //                 v = stream.next() => {
                //                     match v {
                //                         Some(Ok(v)) => {
                //                             sender2.send(jsonrpc::Response {
                //                                 jsonrpc: "2.0",
                //                                 id: id.clone(),
                //                                 result: ResponseInner::Event(v),
                //                             })
                //                             .await;
                //                         }
                //                         Some(Err(_err)) => {
                //                            #[cfg(feature = "tracing")]
                //                             tracing::error!("Subscription error: {:?}", _err);
                //                         }
                //                         None => {
                //                             break;
                //                         }
                //                     }
                //                 }
                //             }
                //         }
                //     });
                // }

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

    sender
        .send(jsonrpc::Response {
            jsonrpc: "2.0",
            id: req.id,
            result,
        })
        .await;
}
