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
    use pin_project_lite::pin_project;
    use serde_json::Value;

    use crate::{
        internal::{middleware::RequestContext, DynBody, ProcedureTodo},
        BuiltRouter, ExecError,
    };

    // TODO: This should be private or handle the "complete" message. Right now `StreamOrFut` handles it and can easily be overlooked by downstream impl.

    pin_project! {
        #[project = OwnedStreamProj]
        /// TODO
        pub struct OwnedStream<TCtx> {
            arc: Arc<BuiltRouter<TCtx>>,
            #[pin]
            reference: Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>,
            pub id: u32,
        }
    }

    impl<TCtx: 'static> OwnedStream<TCtx> {
        pub(crate) fn new(
            router: Arc<BuiltRouter<TCtx>>,
            ctx: TCtx,
            input: Option<Value>,
            req: RequestContext,
            body: &mut DynBody,
        ) -> Result<Self, u32> {
            let stream: *const _ = match router.subscriptions.store.get(req.path.as_ref()) {
                Some(v) => v,
                None => return Err(req.id),
            };

            let id = req.id;

            // SAFETY: Trust me bro
            let stream =
                unsafe { &*stream }
                    .exec
                    .dyn_call(ctx, input.unwrap_or(Value::Null), req, body);

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
        DynBody, ProcedureStore,
    },
    ExecError,
};

use super::{
    AsyncRuntime, ExecRequestFut, Executor, ExecutorResult, Request, Response, ResponseInner,
    SubscriptionManager,
};

// TODO: Seal the following stuff

/// TODO
//
// TODO: Rename
pub struct TrustMeBro<TCtx: Send + 'static> {
    #[allow(unused)]
    arc: Executor<TCtx>,
    queries: *const ProcedureStore<TCtx>,
    mutations: *const ProcedureStore<TCtx>,
}

impl<TCtx: Send + 'static> TrustMeBro<TCtx> {
    pub fn new(arc: Executor<TCtx>) -> Self {
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
    pub fn execute<M: SubscriptionManager<TCtx>>(
        &self,
        ctx: TCtx,
        req: Request,
        mut subscription_manager: &mut Option<M>,
        body: &mut DynBody,
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
                RequestContext::new(id, ProcedureKind::Query, path),
                input,
                body,
            ),
            Request::Mutation { id, path, input } => ExecRequestFut::exec(
                ctx,
                unsafe { &*self.mutations },
                RequestContext::new(id, ProcedureKind::Mutation, path),
                input,
                body,
            ),
            Request::Subscription { id, path, input } => match subscription_manager {
                Some(subscriptions) => self.exec_subscription(
                    ctx,
                    subscriptions,
                    RequestContext::new(id, ProcedureKind::Subscription, path),
                    input,
                    body,
                ),
                None => ExecutorResult::Response(Response {
                    id,
                    inner: ResponseInner::Error(ExecError::ErrSubscriptionsNotSupported.into()),
                }),
            },
            Request::SubscriptionStop { id } => {
                if let Some(subscriptions) = &mut subscription_manager {
                    subscriptions.abort_subscription(id);
                }

                ExecutorResult::None
            }
        }
    }

    fn exec_subscription<M: SubscriptionManager<TCtx>>(
        &self,
        ctx: TCtx,
        subscription_manager: &mut M,
        req: RequestContext,
        input: Option<Value>,
        body: &mut DynBody,
    ) -> ExecutorResult {
        let mut subscriptions = subscription_manager.subscriptions();

        if subscriptions.contains(&req.id) {
            return ExecutorResult::Response(Response {
                id: req.id,
                inner: ResponseInner::Error(ExecError::ErrSubscriptionDuplicateId.into()),
            });
        }

        let id = req.id;
        match OwnedStream::new(self.arc.router.clone(), ctx, input, req, body) {
            Ok(s) => {
                subscriptions.insert(id);
                drop(subscriptions);

                subscription_manager.queue(s);

                ExecutorResult::None
            }
            Err(id) => ExecutorResult::Response(Response {
                id,
                inner: ResponseInner::Error(ExecError::OperationNotFound.into()),
            }),
        }
    }
}
