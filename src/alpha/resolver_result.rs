use std::future::Future;

use futures::{Stream, StreamExt};
use serde::Serialize;
use specta::Type;

use crate::{
    internal::{LayerResult, ValueOrStream},
    Error, ExecError,
};

// For queries and mutations

pub trait AlphaRequestLayer<TMarker> {
    type Result: Type;

    fn into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub enum AlphaSerializeMarker {}
impl<T> AlphaRequestLayer<AlphaSerializeMarker> for T
where
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Ready(Ok(
            serde_json::to_value(self).map_err(ExecError::SerializingResultErr)?
        )))
    }
}

pub enum AlphaResultMarker {}
impl<T> AlphaRequestLayer<AlphaResultMarker> for Result<T, Error>
where
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Ready(Ok(serde_json::to_value(
            self.map_err(ExecError::ErrResolverError)?,
        )
        .map_err(ExecError::SerializingResultErr)?)))
    }
}

pub enum AlphaFutureSerializeMarker {}
impl<TFut, T> AlphaRequestLayer<AlphaFutureSerializeMarker> for TFut
where
    TFut: Future<Output = T> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Future(Box::pin(async move {
            match self
                .await
                .into_layer_result()?
                .into_value_or_stream()
                .await?
            {
                ValueOrStream::Stream(_) => unreachable!(),
                ValueOrStream::Value(v) => Ok(v),
            }
        })))
    }
}

pub enum AlphaFutureResultMarker {}
impl<TFut, T> AlphaRequestLayer<AlphaFutureResultMarker> for TFut
where
    TFut: Future<Output = Result<T, Error>> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Future(Box::pin(async move {
            match self
                .await
                .into_layer_result()?
                .into_value_or_stream()
                .await?
            {
                ValueOrStream::Stream(_) => unreachable!(),
                ValueOrStream::Value(v) => Ok(v),
            }
        })))
    }
}

// For subscriptions

pub trait AlphaStreamRequestLayer<TMarker> {
    type Result: Type;

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
