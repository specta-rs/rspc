/// Marker types (`SerdeTypeMarker` and `FutureTypeMarker`) hold a `PhantomData` to prevent them being constructed by consumers of this crate.
use std::{future::Future, marker::PhantomData};

use serde::Serialize;
use ts_rs::TS;

use crate::{ExecError, MiddlewareResult};

/// TODO
pub trait ResolverResult<TMarker> {
    fn into_middleware_result(self) -> Result<MiddlewareResult, ExecError>;
}

pub struct SerdeTypeMarker(PhantomData<()>);
impl<TValue> ResolverResult<SerdeTypeMarker> for TValue
where
    TValue: Serialize + TS,
{
    fn into_middleware_result(self) -> Result<MiddlewareResult, ExecError> {
        Ok(MiddlewareResult::Sync(
            serde_json::to_value(self).map_err(ExecError::ErrSerialiseResult)?,
        ))
    }
}

pub struct FutureTypeMarker<TReturnMarker>(PhantomData<TReturnMarker>);
impl<TReturnMarker, TReturn, TFut> ResolverResult<FutureTypeMarker<TReturnMarker>> for TFut
where
    TReturnMarker: 'static,
    TReturn: ResolverResult<TReturnMarker> + Send + Sync,
    TFut: Future<Output = TReturn> + Send + Sync + 'static,
{
    fn into_middleware_result(self) -> Result<MiddlewareResult, ExecError> {
        Ok(MiddlewareResult::Future(Box::pin(async move {
            self.await.into_middleware_result()?.await
        })))
    }
}
