use std::{future::Future, marker::PhantomData};

use crate::{
    internal::{RequestResultData, TypedRequestFuture},
    Error, ExecError,
};

pub trait RequestResult<TMarker> {
    type Data: RequestResultData;

    fn into_request_future(self) -> Result<TypedRequestFuture<Self::Data>, ExecError>;
}

pub struct SerializeMarker(PhantomData<()>);
impl<T> RequestResult<SerializeMarker> for T
where
    T: RequestResultData,
{
    type Data = T;

    fn into_request_future(self) -> Result<TypedRequestFuture<Self::Data>, ExecError> {
        Ok(TypedRequestFuture::Ready(Ok(self)))
    }
}

pub struct ResultMarker(PhantomData<()>);
impl<T> RequestResult<ResultMarker> for Result<T, Error>
where
    T: RequestResultData,
{
    type Data = T;

    fn into_request_future(self) -> Result<TypedRequestFuture<Self::Data>, ExecError> {
        Ok(TypedRequestFuture::Ready(Ok(
            self.map_err(ExecError::ErrResolverError)?
        )))
    }
}

pub struct FutureMarker<TMarker>(PhantomData<TMarker>);
impl<TFut, T, TMarker> RequestResult<FutureMarker<TMarker>> for TFut
where
    TFut: Future<Output = T> + Send + 'static,
    T: RequestResult<TMarker> + Send,
{
    type Data = T::Data;

    fn into_request_future(self) -> Result<TypedRequestFuture<Self::Data>, ExecError> {
        Ok(TypedRequestFuture::Future(Box::pin(async move {
            self.await.into_request_future()?.exec().await
        })))
    }
}
