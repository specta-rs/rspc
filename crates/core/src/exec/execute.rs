use std::{pin::Pin, sync::Arc};

use futures::channel::oneshot;

use crate::{
    body::Body,
    error::ExecError,
    exec::{
        arc_ref::{self, get_subscription, ArcRef},
        request_future::RequestFuture,
        Request, Response, ResponseInner, Task,
    },
    Router,
};

use super::SubscriptionMap;

/// TODO
///
// This means a thread is only spawned by us for subscriptions and by the caller for requests.
// If `execute` was async it would *usually* be spawned by the caller but if it were a subscription it would then be spawned again by us.
pub enum ExecutorResult {
    /// A static response
    Response(Response),
    /// A future that will resolve to a response.
    Future(RequestFuture),
    /// A task that should be queued onto an async runtime.
    Task(Task),
}

// TODO: Move this into `build_router.rs` and turn it into a module with all the other `exec::*` types
impl<TCtx: Send + 'static> Router<TCtx> {
    /// TODO
    ///
    /// A `None` result means the executor has no response to send back to the client.
    /// This usually means the request was a subscription and a task was spawned to handle it.
    /// It should not be treated as an error.
    pub fn execute(
        self: Arc<Self>,
        ctx: TCtx,
        req: Request,
        subs: Option<&mut SubscriptionMap>,
    ) -> Option<ExecutorResult> {
        // TODO
        // TODO: Configurable logging hook
        // #[cfg(feature = "tracing")]
        // tracing::trace!(
        //     "Executing operation '{}' with key '{}' with params {:?}",
        //     kind.to_str(),
        //     procedure_name,
        //     input
        // );

        Some(match req {
            Request::Query(data) => map_fut(data.id, arc_ref::get_query(self, ctx, data)),
            Request::Mutation(data) => map_fut(data.id, arc_ref::get_mutation(self, ctx, data)),
            Request::Subscription(data) => {
                let id = data.id;

                match subs {
                    None => Err(ExecError::ErrSubscriptionsNotSupported),
                    Some(subs) if subs.contains_key(data.id) => {
                        Err(ExecError::ErrSubscriptionDuplicateId)
                    }
                    Some(subs) => match get_subscription(self, ctx, data) {
                        None => Err(ExecError::OperationNotFound),
                        Some(stream) => {
                            let (tx, rx) = oneshot::channel();

                            subs.insert(id, tx);

                            Ok(ExecutorResult::Task(Task {
                                id,
                                stream,
                                done: false,
                                shutdown_rx: Some(rx),
                            }))
                        }
                    },
                }
                .unwrap_or_else(|e| {
                    ExecutorResult::Response(Response {
                        id,
                        inner: ResponseInner::Error(e.into()),
                    })
                })
            }
            Request::SubscriptionStop { id } => match subs {
                None => Err(ExecError::ErrSubscriptionsNotSupported),
                Some(subs) => match subs.shutdown(id) {
                    true => return None,
                    false => Err(ExecError::ErrSubscriptionNotFound),
                },
            }
            .unwrap_or_else(|e| {
                ExecutorResult::Response(Response {
                    id,
                    inner: ResponseInner::Error(e.into()),
                })
            }),
        })
    }
}

fn map_fut(id: u32, fut: Option<ArcRef<Pin<Box<dyn Body + Send>>>>) -> ExecutorResult {
    match fut {
        Some(stream) => ExecutorResult::Future(RequestFuture { id, stream }),
        None => ExecutorResult::Response(Response {
            id,
            inner: ResponseInner::Error(ExecError::OperationNotFound.into()),
        }),
    }
}
