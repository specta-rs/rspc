mod private {
    use std::{
        borrow::Cow,
        collections::HashMap,
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
            exec::{AsyncRuntime, Request, Response, ValueOrError},
            middleware::{ProcedureKind, RequestContext},
            ProcedureStore,
        },
        CompiledRouter, ExecError,
    };

    /// TODO
    pub type SubscriptionMap<R> = HashMap<Cow<'static, str>, <R as AsyncRuntime>::TaskHandle>;

    /// TODO
    pub trait SubscriptionManager<R: AsyncRuntime>: Send + 'static {
        type Map<'a>: DerefMut<Target = SubscriptionMap<R>> + 'a;
        type SendFut<'a>: Future<Output = ()> + Send + 'a;

        /// TODO
        fn subscriptions(&mut self) -> Self::Map<'_>;

        /// TODO
        fn send(&mut self, resp: Response) -> Self::SendFut<'_>;
    }

    /// TODO
    #[derive(Clone)]
    pub enum NoOpSubscriptionManager {}

    impl<R: AsyncRuntime> SubscriptionManager<R> for NoOpSubscriptionManager {
        type Map<'a> = &'a mut SubscriptionMap<R>;
        type SendFut<'a> = Ready<()>;

        fn subscriptions(&mut self) -> Self::Map<'_> {
            // Empty enum is unconstructable so this panics will never be hit.
            unreachable!();
        }

        fn send(&mut self, _: Response) -> Self::SendFut<'_> {
            // Empty enum is unconstructable so this panics will never be hit.
            unreachable!();
        }
    }

    /// TODO
    ///
    // This means a thread is only spawned by us for subscriptions and by the caller for requests.
    // If `execute` was async it would *usually* be spawned by the caller but if it were a subscription it would then be spawned again by us.
    pub enum ExecutorResult<'a> {
        FutureResponse(ExecRequestFut<'a>),
        Response(Response),
        None,
    }

    /// TODO
    pub struct Executor<TCtx: Send + 'static, R: AsyncRuntime> {
        router: Arc<CompiledRouter<TCtx>>,
        phantom: PhantomData<R>,
    }

    impl<TCtx: Send + 'static, R: AsyncRuntime> Clone for Executor<TCtx, R> {
        fn clone(&self) -> Self {
            Self {
                router: self.router.clone(),
                phantom: PhantomData,
            }
        }
    }

    impl<TCtx: Send + 'static, R: AsyncRuntime> Executor<TCtx, R> {
        /// constructs a new [Executor] for your router.
        pub fn new(router: Arc<CompiledRouter<TCtx>>) -> Self {
            Self {
                router,
                phantom: PhantomData,
            }
        }

        /// TODO
        ///
        /// WARNING: The response to a batch WILL NOT match the order of the requests in the batch.
        /// This is done for performance reasons and isn't something a proper client should need.
        /// All non responses will be ignored so the response may not be the same length as the request.
        pub async fn execute_batch<M>(
            &self,
            ctx: TCtx,
            reqs: Vec<Request>,
            subscriptions: Option<M>,
        ) -> Vec<Response>
        where
            TCtx: Clone,
            M: SubscriptionManager<R> + Clone,
        {
            // TODO: Probs catch panics so they don't take out the whole batch

            let futs = FuturesUnordered::new();
            let mut resps = Vec::with_capacity(reqs.len());
            for req in reqs {
                match self.execute(ctx.clone(), req, subscriptions.clone()) {
                    ExecutorResult::FutureResponse(fut) => {
                        futs.push(fut);
                    }
                    ExecutorResult::Response(resp) => {
                        resps.push(resp);
                    }
                    ExecutorResult::None => {}
                }
            }

            resps.append(&mut futs.collect::<Vec<_>>().await);
            resps
        }

        /// TODO
        ///
        /// A `None` result means the executor has no response to send back to the client.
        /// This usually means the request was a subscription and a task was spawned to handle it.
        /// It should not be treated as an error.
        pub fn execute<M: SubscriptionManager<R>>(
            &self,
            ctx: TCtx,
            req: Request,
            subscription_manager: Option<M>,
        ) -> ExecutorResult<'_> {
            // TODO
            // #[cfg(feature = "tracing")]
            // tracing::debug!(
            //     "Executing operation '{}' with key '{}' with params {:?}",
            //     kind.to_str(),
            //     procedure_name,
            //     input
            // );

            match req {
                Request::Query { path, input } => ExecRequestFut::exec(
                    ctx,
                    &self.router.queries,
                    RequestContext {
                        kind: ProcedureKind::Query,
                        path,
                        _priv: (),
                    },
                    input,
                ),
                Request::Mutation { path, input } => ExecRequestFut::exec(
                    ctx,
                    &self.router.mutations,
                    RequestContext {
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
                            kind: ProcedureKind::Subscription,
                            path,
                            _priv: (),
                        },
                        id,
                        input,
                    ),
                    None => ExecutorResult::Response(Response::Response {
                        path,
                        result: ValueOrError::Error(ExecError::ErrSubscriptionsNotSupported.into()),
                    }),
                },
                Request::SubscriptionStop { id } => {
                    match subscription_manager {
                        Some(mut subscriptions) => {
                            if let Some(task) = subscriptions.subscriptions().remove(id.as_ref()) {
                                R::cancel_task(task);
                            }
                        }
                        None => {}
                    }

                    ExecutorResult::None
                }
            }
        }

        fn exec_subscription<M: SubscriptionManager<R>>(
            &self,
            ctx: TCtx,
            mut subscription_manager: M,
            req: RequestContext,
            id: Cow<'static, str>,
            input: Option<Value>,
        ) -> ExecutorResult<'_> {
            let mut subscriptions = subscription_manager.subscriptions();

            if subscriptions.contains_key(id.as_ref()) {
                return ExecutorResult::Response(Response::Response {
                    path: req.path,
                    result: ValueOrError::Error(ExecError::ErrSubscriptionDuplicateId.into()),
                });
            }

            match self.router.subscriptions.store.get(req.path.as_ref()) {
                Some(_) => {}
                None => {
                    return ExecutorResult::Response(Response::Response {
                        path: req.path,
                        result: ValueOrError::Error(ExecError::OperationNotFound.into()),
                    })
                }
            }

            let router = self.router.clone();

            let response: Arc<Mutex<(Option<M>, Option<Waker>)>> =
                Arc::new(Mutex::new((None, None)));
            let task_handle = R::spawn({
                let response = response.clone();
                let id = id.clone();

                async move {
                    // This is the receiver side of the manual oneshot.
                    let mut subscription_manager = {
                        poll_fn(|cx| {
                            let mut response = response.lock().unwrap();

                            if let Some(resp) = response.0.take() {
                                return Poll::Ready(resp);
                            }

                            response.1 = Some(cx.waker().clone());

                            Poll::Pending
                        })
                        .await
                    };

                    let op = router
                        .subscriptions
                        .store
                        .get(req.path.as_ref())
                        .expect("Fatal rspc error: An `&T` was modified.");

                    let mut stream = op.exec.dyn_call(ctx, input.unwrap_or(Value::Null), req);

                    loop {
                        while let Some(result) = stream.next().await {
                            let (Ok(result) | Err(result)) = result
                                .map(|v| ValueOrError::Value(v))
                                .map_err(|err| ValueOrError::Error(err.into()));

                            subscription_manager.send(Response::Event {
                                id: id.clone(),
                                result,
                            });
                        }

                        subscription_manager.subscriptions().remove(id.as_ref());

                        // TODO: Inform the frontend it has been shutdown so it can be unregistered
                    }
                }
            });

            subscriptions.insert(id, task_handle);
            drop(subscriptions);

            // TODO: Break out into unit tested primitive
            // Send the `subscription_manager` to the handler thread. This is basically a manually implemented oneshot so we are runtime agnostic.
            {
                let mut response = response.lock().unwrap();

                if response.0.is_none() {
                    response.0 = Some(subscription_manager);
                }

                if let Some(waker) = &response.1 {
                    waker.wake_by_ref();
                }
            }

            ExecutorResult::None
        }
    }

    pub struct ExecRequestFut<'a>(
        Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send + 'a>>,
        Option<Cow<'static, str>>,
    );

    impl<'a> ExecRequestFut<'a> {
        pub fn exec<TCtx: 'static>(
            ctx: TCtx,
            procedures: &'a ProcedureStore<TCtx>,
            req: RequestContext,
            input: Option<Value>,
        ) -> ExecutorResult {
            match procedures.store.get(req.path.as_ref()) {
                Some(procedure) => {
                    let path = req.path.clone();

                    ExecutorResult::FutureResponse(Self(
                        procedure
                            .exec
                            .dyn_call(ctx, input.unwrap_or(Value::Null), req),
                        Some(path),
                    ))
                }
                None => ExecutorResult::Response(Response::Response {
                    path: req.path,
                    result: ValueOrError::Error(ExecError::OperationNotFound.into()),
                }),
            }
        }
    }

    impl<'a> Future for ExecRequestFut<'a> {
        type Output = Response;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.0.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(result))) => Poll::Ready(Response::Response {
                    path: self
                        .1
                        .take()
                        .expect("fatal rspc error: 'ExecRequestFut' polled after completion!"),
                    result: ValueOrError::Value(result),
                }),
                Poll::Ready(Some(Err(err))) => Poll::Ready(Response::Response {
                    path: self
                        .1
                        .take()
                        .expect("fatal rspc error: 'ExecRequestFut' polled after completion!"),
                    result: ValueOrError::Error(err.into()),
                }),
                Poll::Ready(None) => Poll::Ready(Response::Response {
                    path: self
                        .1
                        .take()
                        .expect("fatal rspc error: 'ExecRequestFut' polled after completion!"),
                    result: ValueOrError::Error(ExecError::ErrStreamEmpty.into()),
                }),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use private::{
    Executor, ExecutorResult, NoOpSubscriptionManager, SubscriptionManager, SubscriptionMap,
};

#[cfg(not(feature = "unstable"))]
pub(crate) use private::{
    Executor, ExecutorResult, NoOpSubscriptionManager, SubscriptionManager, SubscriptionMap,
};
