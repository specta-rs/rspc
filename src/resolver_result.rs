use std::{future::Future, marker::PhantomData};

use serde::Serialize;
use specta::Type;

use crate::{internal::LayerResult, Error, ExecError};

pub trait RequestLayer<TMarker> {
    type Result: Type;

    fn into_layer_result(self) -> Result<LayerResult, ExecError>;
}

pub struct SerializeMarker(PhantomData<()>);
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

pub struct ResultMarker(PhantomData<()>);
impl<T> RequestLayer<ResultMarker> for Result<T, Error>
where
    T: Serialize + Type,
{
    type Result = T;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::Ready(Ok(serde_json::to_value(
            self.map_err(|err| ExecError::ErrResolverError(err))?,
        )
        .map_err(ExecError::SerializingResultErr)?)))
    }
}

pub struct FutureMarker<TMarker>(PhantomData<TMarker>);
impl<TFut, T, TMarker> RequestLayer<FutureMarker<TMarker>> for TFut
where
    TFut: Future<Output = T> + Send + 'static,
    T: RequestLayer<TMarker> + Send,
{
    type Result = T::Result;

    fn into_layer_result(self) -> Result<LayerResult, ExecError> {
        Ok(LayerResult::FutureStreamOrValue(Box::pin(async move {
            self.await.into_layer_result()?.into_value().await
        })))
    }
}
