use std::{future::Future, marker::PhantomData};

use serde::Serialize;
use specta::Type;

use crate::{Error, ExecError, LayerResult};

pub trait IntoLayerResult<TMarker> {
    type Result: Type;

    fn into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub struct SerializeMarker(PhantomData<()>);
impl<T> IntoLayerResult<SerializeMarker> for T
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

pub struct ResultMarker(PhantomData<()>);
impl<T, TErr> IntoLayerResult<ResultMarker> for Result<T, TErr>
where
    T: Serialize + Type,
    TErr: Into<Error>,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Ready(Ok(serde_json::to_value(self.map_err(
            |err| ExecError::ErrResolverError(err.into()),
        )?)
        .map_err(ExecError::SerializingResultErr)?)))
    }
}

pub struct FutureMarker<TMarker>(PhantomData<TMarker>);
impl<TFut, T, TMarker> IntoLayerResult<FutureMarker<TMarker>> for TFut
where
    TFut: Future<Output = T> + Send + 'static,
    T: IntoLayerResult<TMarker> + Send,
{
    type Result = T::Result;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::FutureStreamOrValue(Box::pin(async move {
            self.await.into_layer_result()?.into_stream_or_value().await
        })))
    }
}
