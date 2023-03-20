use std::{future::Future, marker::PhantomData};

use futures::{Stream, StreamExt};
use serde::Serialize;
use specta::Type;

use crate::{
    internal::{LayerResult, ValueOrStream},
    Error, ExecError,
};

/// For either
pub trait AnyRequestLayer<TMarker> {
    type Result: Type;

    // TODO: Rename
    fn any_into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub struct RequestLayerMarker<T>(PhantomData<T>);
impl<T: RequestLayer<TMarker>, TMarker> AnyRequestLayer<RequestLayerMarker<TMarker>> for T {
    type Result = T::Result;

    fn any_into_layer_result(self) -> Result<LayerResult, ExecError> {
        RequestLayer::into_layer_result(self)
    }
}

pub struct StreamLayerMarker<T>(PhantomData<T>);
impl<T: StreamRequestLayer<TMarker>, TMarker> AnyRequestLayer<StreamLayerMarker<TMarker>> for T {
    type Result = T::Result;

    fn any_into_layer_result(self) -> Result<LayerResult, ExecError> {
        StreamRequestLayer::into_layer_result(self)
    }
}

// For queries and mutations

pub trait RequestLayer<TMarker> {
    type Result: Type;

    fn into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub enum SerializeMarker {}
impl<T> RequestLayer<SerializeMarker> for T
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

pub enum ResultMarker {}
impl<T> RequestLayer<ResultMarker> for Result<T, Error>
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

pub enum FutureSerializeMarker {}
impl<TFut, T> RequestLayer<FutureSerializeMarker> for TFut
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

pub enum FutureResultMarker {}
impl<TFut, T> RequestLayer<FutureResultMarker> for TFut
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

pub trait StreamRequestLayer<TMarker> {
    type Result: Type;

    fn into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub enum StreamMarker {}
impl<TStream, T> StreamRequestLayer<StreamMarker> for TStream
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

pub enum ResultStreamMarker {}
impl<TStream, T> StreamRequestLayer<ResultStreamMarker> for Result<TStream, Error>
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

pub enum FutureStreamMarker {}
impl<TFut, TStream, T> StreamRequestLayer<FutureStreamMarker> for TFut
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

pub enum FutureResultStreamMarker {}
impl<TFut, TStream, T> StreamRequestLayer<FutureResultStreamMarker> for TFut
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
