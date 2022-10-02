use std::{future::Future, marker::PhantomData};

use serde::Serialize;
use specta::Type;

use crate::{internal::RequestFuture, Error, ExecError};

pub trait RequestLayer<TMarker> {
    type Result: Type;

    fn into_request_future(self) -> Result<RequestFuture, ExecError>;
}

pub struct SerializeMarker(PhantomData<()>);
impl<T> RequestLayer<SerializeMarker> for T
where
    T: Serialize + Type,
{
    type Result = T;

    fn into_request_future(self) -> Result<RequestFuture, ExecError> {
        Ok(RequestFuture::Ready(Ok(
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

    fn into_request_future(self) -> Result<RequestFuture, ExecError> {
        Ok(RequestFuture::Ready(Ok(serde_json::to_value(
            self.map_err(ExecError::ErrResolverError)?,
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

    fn into_request_future(self) -> Result<RequestFuture, ExecError> {
        Ok(RequestFuture::Future(Box::pin(async move {
            self.await.into_request_future()?.exec().await
        })))
    }
}
