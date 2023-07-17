mod private {
    use std::{
        borrow::Cow,
        future::Future,
        marker::PhantomData,
        mem::transmute,
        ops::Deref,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
    };

    use futures::Stream;
    use pin_project::pin_project;
    use serde_json::Value;

    use crate::{
        internal::{middleware::RequestContext, ProcedureTodo},
        BuiltRouter, ExecError,
    };

    /// TODO
    #[pin_project(project = OwnedStreamProj)]
    pub struct OwnedStream<TCtx: 'static> {
        arc: Arc<BuiltRouter<TCtx>>,
        #[pin]
        pub(crate) reference: Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>,
        pub(crate) id: u32,
    }

    impl<TCtx: 'static> OwnedStream<TCtx> {
        pub(crate) fn new(
            router: Arc<BuiltRouter<TCtx>>,
            ctx: TCtx,
            input: Option<Value>,
            req: RequestContext,
        ) -> Result<Self, u32> {
            let stream: *const _ = match router.subscriptions.store.get(req.path.as_ref()) {
                Some(v) => v,
                None => return Err(req.id),
            };

            let id = *&req.id;

            // SAFETY: Trust me bro
            let stream = unsafe { &*stream }
                .exec
                .dyn_call(ctx, input.unwrap_or(Value::Null), req);

            Ok(Self {
                arc: router,
                reference: stream,
                id,
            })
        }
    }

    impl<TCtx: 'static> Stream for OwnedStream<TCtx> {
        type Item = Result<Value, ExecError>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.project().reference.poll_next(cx)
        }
    }
}

use std::{
    borrow::Cow,
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::Stream;
#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use private::OwnedStream;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::OwnedStream;
use serde_json::Value;

use crate::{
    internal::{
        middleware::{ProcedureKind, RequestContext},
        ProcedureStore,
    },
    ExecError,
};

use super::{
    AsyncRuntime, ExecRequestFut, Executor, ExecutorResult, Request, Response, SubscriptionManager,
    ValueOrError,
};

// TODO: Seal the following stuff

/// TODO
//
// TODO: Rename
pub struct TrustMeBro<TCtx: Send + 'static, R: AsyncRuntime> {
    #[allow(unused)]
    arc: Executor<TCtx, R>,
    queries: *const ProcedureStore<TCtx>,
    mutations: *const ProcedureStore<TCtx>,
}

impl<TCtx: Send + 'static, R: AsyncRuntime> TrustMeBro<TCtx, R> {
    pub fn new(arc: Executor<TCtx, R>) -> Self {
        Self {
            queries: &arc.router.queries,
            mutations: &arc.router.mutations,
            arc,
        }
    }

    /// TODO
    ///
    /// A `None` result means the executor has no response to send back to the client.
    /// This usually means the request was a subscription and a task was spawned to handle it.
    /// It should not be treated as an error.
    pub fn execute<M: SubscriptionManager<R, TCtx>>(
        &self,
        ctx: TCtx,
        req: Request,
        mut subscription_manager: &mut Option<M>,
    ) -> ExecutorResult {
        // TODO
        // #[cfg(feature = "tracing")]
        // tracing::debug!(
        //     "Executing operation '{}' with key '{}' with params {:?}",
        //     kind.to_str(),
        //     procedure_name,
        //     input
        // );

        match req {
            Request::Query { id, path, input } => ExecRequestFut::exec(
                ctx,
                unsafe { &*self.queries },
                RequestContext {
                    id,
                    kind: ProcedureKind::Query,
                    path,
                    _priv: (),
                },
                input,
            ),
            Request::Mutation { id, path, input } => ExecRequestFut::exec(
                ctx,
                unsafe { &*self.mutations },
                RequestContext {
                    id,
                    kind: ProcedureKind::Mutation,
                    path,
                    _priv: (),
                },
                input,
            ),
            Request::Subscription { id, path, input } => match subscription_manager {
                Some(subscriptions) => self.exec_subscription(
                    ctx,
                    subscriptions,
                    RequestContext {
                        id,
                        kind: ProcedureKind::Subscription,
                        path,
                        _priv: (),
                    },
                    input,
                ),
                None => ExecutorResult::Response(Response {
                    id,
                    result: ValueOrError::Error(ExecError::ErrSubscriptionsNotSupported.into()),
                }),
            },
            Request::SubscriptionStop { id } => {
                if let Some(subscriptions) = &mut subscription_manager {
                    if let Some(task) = subscriptions.subscriptions().remove(&id) {
                        R::cancel_task(task);
                    }
                }

                ExecutorResult::None
            }
        }
    }

    fn exec_subscription<M: SubscriptionManager<R, TCtx>>(
        &self,
        ctx: TCtx,
        subscription_manager: &mut M,
        req: RequestContext,
        input: Option<Value>,
    ) -> ExecutorResult {
        let subscriptions = subscription_manager.subscriptions();

        if subscriptions.contains_key(&req.id) {
            return ExecutorResult::Response(Response {
                id: req.id,
                result: ValueOrError::Error(ExecError::ErrSubscriptionDuplicateId.into()),
            });
        }

        match OwnedStream::new(self.arc.router.clone(), ctx, input, req) {
            Ok(s) => {
                // subscriptions.insert(id, task_handle); // TODO
                drop(subscriptions);

                subscription_manager.queue(s.id, s);

                ExecutorResult::None
            }
            Err(id) => ExecutorResult::Response(Response {
                id,
                result: ValueOrError::Error(ExecError::OperationNotFound.into()),
            }),
        }
    }
}
