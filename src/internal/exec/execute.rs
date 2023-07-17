mod private {
    use std::{
        borrow::Cow,
        collections::{HashMap, HashSet},
        convert::Infallible,
        future::{Future, Ready},
        marker::PhantomData,
        ops::DerefMut,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
    };

    use futures::{future::poll_fn, stream::FuturesUnordered, Stream, StreamExt};

    use serde_json::Value;

    use crate::{
        internal::{
            exec::{AsyncRuntime, OwnedStream, Request, Response, ResponseInner},
            middleware::{ProcedureKind, RequestContext},
            FutureValueOrStream, ProcedureStore, ProcedureTodo,
        },
        BuiltRouter, ExecError,
    };

    /// Map for subscription id to task handle.
    /// This is used for shutting down subscriptions.
    pub type SubscriptionSet = HashSet<u32>;

    /// TODO
    pub trait SubscriptionManager<TCtx> {
        type Set<'m>: DerefMut<Target = SubscriptionSet> + 'm
        where
            Self: 'm;

        /// TODO
        fn queue(&mut self, id: u32, stream: OwnedStream<TCtx>);

        /// TODO
        fn subscriptions(&mut self) -> Self::Set<'_>;

        /// TODO
        fn abort_subscription(&mut self, id: u32);
    }

    /// TODO
    #[derive(Clone)]
    pub enum NoOpSubscriptionManager {}

    impl<TCtx> SubscriptionManager<TCtx> for NoOpSubscriptionManager {
        type Set<'a> = &'a mut SubscriptionSet;

        fn queue(&mut self, _id: u32, _task: OwnedStream<TCtx>) {
            // Empty enum is unconstructable so this panics will never be hit.
            unreachable!();
        }

        fn subscriptions(&mut self) -> Self::Set<'_> {
            // Empty enum is unconstructable so this panics will never be hit.
            unreachable!();
        }

        fn abort_subscription(&mut self, _id: u32) {
            // Empty enum is unconstructable so this panics will never be hit.
            unreachable!();
        }
    }

    /// TODO
    ///
    // This means a thread is only spawned by us for subscriptions and by the caller for requests.
    // If `execute` was async it would *usually* be spawned by the caller but if it were a subscription it would then be spawned again by us.
    pub enum ExecutorResult {
        FutureResponse(ExecRequestFut),
        Response(Response),
        None,
    }

    /// TODO
    pub struct Executor<TCtx> {
        // TODO: Not `pub`
        pub(crate) router: Arc<BuiltRouter<TCtx>>,
    }

    impl<TCtx: Send + 'static> Clone for Executor<TCtx> {
        fn clone(&self) -> Self {
            Self {
                router: self.router.clone(),
            }
        }
    }

    impl<TCtx: Send + 'static> Executor<TCtx> {
        /// constructs a new [Executor] for your router.
        pub fn new(router: Arc<BuiltRouter<TCtx>>) -> Self {
            Self { router }
        }

        /// TODO
        ///
        /// WARNING: The response to a batch WILL NOT match the order of the requests in the batch.
        /// This is done for performance reasons and isn't something a proper client should need.
        /// All non responses will be ignored so the response may not be the same length as the request.
        ///
        // WARNING: The result of this function will not contain all requests.
        // Your expected to use the `queue` fn to push them onto the runtime and handle them when completed
        pub(crate) fn execute_batch<'a, M>(
            &'a self,
            ctx: &TCtx,
            reqs: Vec<Request>,
            subscriptions: &mut Option<M>,
            mut queue: impl FnMut(ExecRequestFut) + 'a,
        ) -> Vec<Response>
        where
            TCtx: Clone,
            M: SubscriptionManager<TCtx>,
        {
            let mut resps = Vec::with_capacity(reqs.len());

            // TODO: Probs catch panics so they don't take out the whole batch

            for req in reqs {
                match self.execute(ctx.clone(), req, subscriptions) {
                    ExecutorResult::FutureResponse(fut) => queue(fut),
                    ExecutorResult::Response(resp) => {
                        resps.push(resp);
                    }
                    ExecutorResult::None => {}
                }
            }

            resps
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
                    &self.router.queries,
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
                    &self.router.mutations,
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
        ) -> ExecutorResult {
            let mut subscriptions = subscription_manager.subscriptions();

            if subscriptions.contains(&req.id) {
                return ExecutorResult::Response(Response {
                    id: req.id,
                    inner: ResponseInner::Error(ExecError::ErrSubscriptionDuplicateId.into()),
                });
            }

            let id = req.id;
            match OwnedStream::new(self.router.clone(), ctx, input, req) {
                Ok(s) => {
                    subscriptions.insert(id);
                    drop(subscriptions);

                    subscription_manager.queue(id, s);

                    ExecutorResult::None
                }
                Err(id) => ExecutorResult::Response(Response {
                    id,
                    inner: ResponseInner::Error(ExecError::OperationNotFound.into()),
                }),
            }
        }
    }

    pub struct ExecRequestFut {
        stream: Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>,
        pub(crate) id: u32,
    }

    impl ExecRequestFut {
        pub fn exec<TCtx: 'static>(
            ctx: TCtx,
            procedures: *const ProcedureStore<TCtx>,
            req: RequestContext,
            input: Option<Value>,
        ) -> ExecutorResult {
            // TODO: This unsafe is not coupled to the Arc which is bad
            match unsafe { &*procedures }.store.get(req.path.as_ref()) {
                Some(procedure) => ExecutorResult::FutureResponse(Self {
                    id: req.id,
                    stream: procedure
                        .exec
                        .dyn_call(ctx, input.unwrap_or(Value::Null), req),
                }),
                None => ExecutorResult::Response(Response {
                    id: req.id,
                    inner: ResponseInner::Error(ExecError::OperationNotFound.into()),
                }),
            }
        }
    }

    impl Future for ExecRequestFut {
        type Output = Response;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(result))) => Poll::Ready(Response {
                    id: self.id,
                    inner: ResponseInner::Value(result),
                }),
                Poll::Ready(Some(Err(err))) => Poll::Ready(Response {
                    id: self.id,
                    inner: ResponseInner::Error(err.into()),
                }),
                Poll::Ready(None) => Poll::Ready(Response {
                    id: self.id,
                    inner: ResponseInner::Error(ExecError::ErrStreamEmpty.into()),
                }),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

pub(crate) use private::ExecRequestFut;

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use private::{
    Executor, ExecutorResult, NoOpSubscriptionManager, SubscriptionManager, SubscriptionSet,
};

#[cfg(not(feature = "unstable"))]
pub(crate) use private::{
    Executor, ExecutorResult, NoOpSubscriptionManager, SubscriptionManager, SubscriptionSet,
};
