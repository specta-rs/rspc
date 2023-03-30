use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use pin_project::pin_project;
use serde::Serialize;
use specta::Type;

use crate::{
    internal::{LayerResult, ValueOrStream},
    Error, ExecError,
};

// For queries and mutations

pub trait AlphaRequestLayer<TMarker> {
    type Result: Type;
    type Fut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static;

    // TODO: Rename func
    fn into_layer_result(self) -> Self::Fut;
}

pub enum AlphaSerializeMarker {}
impl<T> AlphaRequestLayer<AlphaSerializeMarker> for T
where
    T: Serialize + Type,
{
    type Result = T;
    type Fut = Ready<Result<ValueOrStream, ExecError>>;

    fn into_layer_result(self) -> Self::Fut {
        ready(
            serde_json::to_value(self)
                .map(ValueOrStream::Value)
                .map_err(ExecError::SerializingResultErr),
        )
    }
}

pub enum AlphaResultMarker {}
impl<T> AlphaRequestLayer<AlphaResultMarker> for Result<T, Error>
where
    T: Serialize + Type,
{
    type Result = T;
    type Fut = Ready<Result<ValueOrStream, ExecError>>;

    fn into_layer_result(self) -> Self::Fut {
        ready(self.map_err(ExecError::ErrResolverError).and_then(|v| {
            serde_json::to_value(v)
                .map(ValueOrStream::Value)
                .map_err(ExecError::SerializingResultErr)
        }))
    }
}

pub enum AlphaFutureSerializeMarker {}
impl<TFut, T> AlphaRequestLayer<AlphaFutureSerializeMarker> for TFut
where
    TFut: Future<Output = T> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Result = T;
    type Fut = FutureSerializeFuture<TFut, T>;

    fn into_layer_result(self) -> Self::Fut {
        FutureSerializeFuture(self, PhantomData)
    }
}

#[pin_project(project = FutureSerializeFutureProj)]
pub struct FutureSerializeFuture<TFut, T>(#[pin] TFut, PhantomData<T>);

impl<TFut, T> Future for FutureSerializeFuture<TFut, T>
where
    TFut: Future<Output = T> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Output = Result<ValueOrStream, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            Poll::Ready(v) => Poll::Ready(
                serde_json::to_value(v)
                    .map(ValueOrStream::Value)
                    .map_err(ExecError::SerializingResultErr),
            ),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub enum AlphaFutureResultMarker {}
impl<TFut, T> AlphaRequestLayer<AlphaFutureResultMarker> for TFut
where
    TFut: Future<Output = Result<T, Error>> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Result = T;
    type Fut = FutureSerializeResultFuture<TFut, T>;

    fn into_layer_result(self) -> Self::Fut {
        FutureSerializeResultFuture(self, PhantomData)
    }
}

#[pin_project(project = FutureSerializeResultFutureProj)]
pub struct FutureSerializeResultFuture<TFut, T>(#[pin] TFut, PhantomData<T>);

impl<TFut, T> Future for FutureSerializeResultFuture<TFut, T>
where
    TFut: Future<Output = Result<T, Error>> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Output = Result<ValueOrStream, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            Poll::Ready(v) => Poll::Ready(v.map_err(ExecError::ErrResolverError).and_then(|v| {
                serde_json::to_value(v)
                    .map(ValueOrStream::Value)
                    .map_err(ExecError::SerializingResultErr)
            })),
            Poll::Pending => Poll::Pending,
        }
    }
}

// For subscriptions

pub trait AlphaStreamRequestLayer<TMarker> {
    type Result: Type;
    // type Stream; // TODO: make this work

    fn into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub enum AlphaStreamMarker {}
impl<TStream, T> AlphaStreamRequestLayer<AlphaStreamMarker> for TStream
where
    TStream: Stream<Item = T> + Send + Sync + 'static,
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Stream(Box::pin(self.map(|v| {
            serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
        }))))
    }
}

pub enum AlphaResultStreamMarker {}
impl<TStream, T> AlphaStreamRequestLayer<AlphaResultStreamMarker> for Result<TStream, Error>
where
    TStream: Stream<Item = T> + Send + Sync + 'static,
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Stream(Box::pin(
            self.map_err(ExecError::ErrResolverError)?
                .map(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr)),
        )))
    }
}

pub enum AlphaFutureStreamMarker {}
impl<TFut, TStream, T> AlphaStreamRequestLayer<AlphaFutureStreamMarker> for TFut
where
    TFut: Future<Output = TStream> + Send + 'static,
    TStream: Stream<Item = T> + Send + Sync + 'static,
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::FutureValueOrStream(Box::pin(async move {
            Ok(ValueOrStream::Stream(Box::pin(self.await.map(|v| {
                serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
            }))))
        })))
    }
}

pub enum AlphaFutureResultStreamMarker {}
impl<TFut, TStream, T> AlphaStreamRequestLayer<AlphaFutureResultStreamMarker> for TFut
where
    TFut: Future<Output = Result<TStream, Error>> + Send + 'static,
    TStream: Stream<Item = T> + Send + Sync + 'static,
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::FutureValueOrStream(Box::pin(async move {
            Ok(ValueOrStream::Stream(Box::pin(
                self.await
                    .map_err(ExecError::ErrResolverError)?
                    .map(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr)),
            )))
        })))
    }
}
