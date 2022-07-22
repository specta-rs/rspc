/// Marker types (`SerdeTypeMarker` and `FutureTypeMarker`) hold a `PhantomData` to prevent them being constructed by consumers of this crate.
use std::{future::Future, marker::PhantomData};

use serde::Serialize;
use ts_rs::TS;

use crate::{Error, ExecError, MiddlewareResult, TypeDef};

/// TODO
pub trait ResolverResult<TMarker> {
    fn into_middleware_result(self) -> Result<MiddlewareResult, ExecError>;

    fn type_def<TArg: TS, TLayerArg: TS>() -> TypeDef;
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

    fn type_def<TArg: TS, TLayerArg: TS>() -> TypeDef {
        TypeDef::new::<TArg, TLayerArg, TValue>()
    }
}

pub struct FutureTypeMarker<TReturnMarker>(PhantomData<TReturnMarker>);
impl<TReturnMarker, TReturn, TFut> ResolverResult<FutureTypeMarker<TReturnMarker>> for TFut
where
    TReturnMarker: 'static,
    TReturn: ResolverResult<TReturnMarker> + Send,
    TFut: Future<Output = TReturn> + Send + 'static,
{
    fn into_middleware_result(self) -> Result<MiddlewareResult, ExecError> {
        Ok(MiddlewareResult::Future(Box::pin(async move {
            self.await.into_middleware_result()?.await
        })))
    }

    fn type_def<TArg: TS, TLayerArg: TS>() -> TypeDef {
        TReturn::type_def::<TArg, TLayerArg>()
    }
}

pub struct ResultTypeMarker<TReturnMarker>(PhantomData<TReturnMarker>);
impl<TValue, TErr> ResolverResult<ResultTypeMarker<TErr>> for Result<TValue, TErr>
where
    TValue: Serialize + TS,
    TErr: Into<Error>,
{
    fn into_middleware_result(self) -> Result<MiddlewareResult, ExecError> {
        match self {
            Ok(value) => Ok(MiddlewareResult::Sync(
                serde_json::to_value(value).map_err(ExecError::ErrSerialiseResult)?,
            )),
            Err(err) => Err(ExecError::ErrResolverError(err.into())),
        }
    }

    fn type_def<TArg: TS, TLayerArg: TS>() -> TypeDef {
        TypeDef::new::<TArg, TLayerArg, TValue>()
    }
}
