use std::{
    future::Future,
    marker::PhantomData,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use async_recursion::async_recursion;
use futures::Stream;
use serde_json::Value;

use crate::{ConcreteArg, ExecError};

/// TODO
pub(crate) type MiddlewareChainBase<TCtx> =
    Box<dyn Fn(TCtx, ConcreteArg) -> Result<MiddlewareResult, ExecError> + Send + Sync>;

/// TODO
pub(crate) type OperationHandler<TLayerCtx> =
    Box<dyn Fn(TLayerCtx, ConcreteArg) -> Result<MiddlewareResult, ExecError> + Send + Sync>;

/// TODO
pub(crate) type MiddlewareChain<TCtx, TLayerCtx> =
    Box<dyn Fn(OperationHandler<TLayerCtx>) -> MiddlewareChainBase<TCtx> + Send + Sync>;

pub enum MaybeDone<T> {
    Poll(T),
    Gone,
}

/// TODO
pub enum MiddlewareResult {
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    FutureMiddlewareResult(
        Pin<Box<dyn Future<Output = Result<MiddlewareResult, ExecError>> + Send>>,
    ),
    Sync(Value),
    // I hate this
    // PollFuture(
    //     MaybeDone<Pin<Box<dyn Future<Output = Result<MiddlewareResult, ExecError>> + Send>>>,
    // ),
    Gone,
}

pub enum StreamOrValue {
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    Value(Value),
}

impl MiddlewareResult {
    /// TODO: This method is midly cringe
    #[async_recursion] // TODO: Remove this cause it does allocations which aren't controlled by us and we should be able to work around needing.
    pub async fn to_stream_or_value(self) -> Result<StreamOrValue, ExecError> {
        match self {
            MiddlewareResult::Stream(stream) => Ok(StreamOrValue::Stream(stream)),
            MiddlewareResult::Future(fut) => Ok(StreamOrValue::Value(fut.await?)),
            MiddlewareResult::Sync(value) => Ok(StreamOrValue::Value(value)),
            MiddlewareResult::FutureMiddlewareResult(future) => {
                future.await?.to_stream_or_value().await
            }
            MiddlewareResult::Gone => unreachable!(),
        }
    }
}

impl Future for MiddlewareResult {
    type Output = Result<Value, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.get_mut();
        loop {
            break match this {
                MiddlewareResult::Stream(_) => unimplemented!(), // TODO: Make this conditional impossible in type system or handle it.
                MiddlewareResult::Future(fut) => Pin::new(fut).poll(cx),
                MiddlewareResult::Sync(_) => {
                    let v = match mem::replace(this, MiddlewareResult::Gone) {
                        MiddlewareResult::Sync(v) => v,
                        _ => unreachable!(),
                    };

                    Poll::Ready(Ok(v))
                }
                MiddlewareResult::FutureMiddlewareResult(fut) => {
                    match Pin::new(fut).poll(cx) {
                        Poll::Ready(value) => {
                            let _ = mem::replace(this, value?);
                            continue;
                        }
                        Poll::Pending => Poll::Pending,
                    }

                    // let x = match mem::replace(this, MiddlewareResult::Gone) {
                    //     MiddlewareResult::FutureMiddlewareResult(x) => x,
                    //     _ => unreachable!(),
                    // };
                    // println!("BRUH");
                    // mem::replace(this, MiddlewareResult::PollFuture(MaybeDone::Poll(x)));
                    // continue;

                    // let fut = Pin::new(future);
                    // let fut = fut.get_mut();
                    // this = fut;

                    // let y = Pin::new(fut).poll(cx);

                    // unimplemented!();
                }
                // MiddlewareResult::PollFuture(fut) => {
                //     // Pin::new(fut).poll(cx)

                //     // let fut = match mem::replace(this, MiddlewareResult::Gone) {
                //     //     MiddlewareResult::PollFuture(fut) => fut,
                //     //     _ => unreachable!(),
                //     // };
                //     // let fut = Pin::new(fut);
                //     // let fut = fut.get_mut();
                //     // this = fut;
                //     // continue;
                // }
                MiddlewareResult::Gone => unreachable!(),
            };
        }
    }
}

/// TODO: Cringe name
pub trait ActualMiddlewareResult<TMarker> {
    fn into_middleware_result(self) -> MiddlewareResult;
}

pub struct ActualMiddlewareResultValueMarker(PhantomData<()>);
impl ActualMiddlewareResult<ActualMiddlewareResultValueMarker> for Value {
    fn into_middleware_result(self) -> MiddlewareResult {
        MiddlewareResult::Sync(self)
    }
}

pub struct ActualMiddlewareResultStreamMarker(PhantomData<()>);
impl<TFut: Stream<Item = Result<Value, ExecError>> + Send + 'static>
    ActualMiddlewareResult<ActualMiddlewareResultStreamMarker> for TFut
{
    fn into_middleware_result(self) -> MiddlewareResult {
        MiddlewareResult::Stream(Box::pin(self))
    }
}
