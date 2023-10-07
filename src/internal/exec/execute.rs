use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::Infallible,
    fmt,
    future::{Future, Ready},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::{future::poll_fn, stream::FuturesUnordered, Stream, StreamExt};

use serde_json::Value;

use crate::{
    internal::{
        exec::{
            arc_ref::{self, get_subscription, ArcRef},
            request_future::RequestFuture,
            Request, Response, ResponseInner, Task,
        },
        exec2::Connection,
        middleware::{ProcedureKind, RequestContext},
        procedure::ProcedureTodo,
        Body, FutureValueOrStream,
    },
    BuiltRouter, ExecError, ProcedureMap,
};

// TODO: The big problem with removing `TCtx` everywhere is that it is required in `Box<dyn DynLayer<TCtx>` which is the thing we must hold to ensure the `unsafe` parts are safe.
// TODO: Just bumping the reference count will ensure it's not unsafely dropped but will also likely result in a memory leak cause without knowing the type one of the request-types can't take care of dropping it's data if it needs to be dropped.

/// TODO
///
// This means a thread is only spawned by us for subscriptions and by the caller for requests.
// If `execute` was async it would *usually* be spawned by the caller but if it were a subscription it would then be spawned again by us.
// TODO: plz make this not-generic it sucks
pub enum ExecutorResult {
    /// A static response
    Response(Response),
    /// A future that will resolve to a response.
    Future(RequestFuture),
    /// A task that should be queued onto an async runtime.
    Task(Task),
    /// A `None` result means the executor has no response to send back to the client.
    /// This usually means the request was a subscription and a task was spawned to handle it.
    /// It should **not** be treated as an error.
    None,
}

// TODO: Move this into `build_router.rs` and turn it into a module with all the other `exec::*` types
impl<TCtx: Send + 'static> BuiltRouter<TCtx> {
    /// TODO
    ///
    /// A `None` result means the executor has no response to send back to the client.
    /// This usually means the request was a subscription and a task was spawned to handle it.
    /// It should not be treated as an error.
    pub fn execute(
        self: &Arc<Self>,
        ctx: TCtx,
        req: Request,
        conn: Option<impl Deref<Target = Connection> + DerefMut>,
    ) -> ExecutorResult {
        // TODO
        // TODO: Configurable logging hook
        // #[cfg(feature = "tracing")]
        // tracing::trace!(
        //     "Executing operation '{}' with key '{}' with params {:?}",
        //     kind.to_str(),
        //     procedure_name,
        //     input
        // );

        match req {
            Request::Query { id, path, input } => map_fut(
                id,
                arc_ref::get_query(
                    self.clone(),
                    ctx,
                    input,
                    RequestContext::new(id, ProcedureKind::Query, path),
                ),
            ),
            Request::Mutation { id, path, input } => map_fut(
                id,
                arc_ref::get_mutation(
                    self.clone(),
                    ctx,
                    input,
                    RequestContext::new(id, ProcedureKind::Mutation, path),
                ),
            ),
            Request::Subscription { id, path, input } => match conn {
                Some(mut conn) => {
                    if conn.subscriptions.contains_key(&id) {
                        return ExecutorResult::Response(Response {
                            id: id,
                            inner: ResponseInner::Error(
                                ExecError::ErrSubscriptionDuplicateId.into(),
                            ),
                        });
                    }

                    match get_subscription(
                        self.clone(),
                        ctx,
                        input,
                        RequestContext::new(id, ProcedureKind::Subscription, path),
                    ) {
                        Some(stream) => ExecutorResult::Task(Task {
                            id,
                            stream,
                            done: 0,
                        }),
                        None => ExecutorResult::Response(Response {
                            id,
                            inner: ResponseInner::Error(ExecError::OperationNotFound.into()),
                        }),
                    }
                }
                None => ExecutorResult::Response(Response {
                    id,
                    inner: ResponseInner::Error(ExecError::ErrSubscriptionsNotSupported.into()),
                }),
            },
            Request::SubscriptionStop { id } => {
                if let Some(mut conn) = conn {
                    conn.subscriptions.remove(&id);
                }

                ExecutorResult::None
            }
        }
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
