//! The future of this code in unsure. It will probs be removed or refactored once we support more than just Axum because all of the feature gating is bad.

use crate::ExecError;

use futures::Future;
#[cfg(not(feature = "workers"))]
use httpz::axum::axum::extract::{FromRequest, RequestParts};
use std::{marker::PhantomData, pin::Pin};

// TODO: Can we avoid needing to box the extractors????
// TODO: Support for up to 16 extractors
// TODO: Debug bounds on `::Rejection` should only happen in the `tracing` feature is enabled
// TODO: Allow async context functions

pub enum TCtxFuncResult<'a, TCtx> {
    Value(Result<TCtx, ExecError>),
    Future(Pin<Box<dyn Future<Output = Result<TCtx, ExecError>> + Send + 'a>>),
}

#[cfg(not(feature = "workers"))]
pub trait TCtxFunc<TCtx, TMarker>: Clone + Send + Sync + 'static
where
    TCtx: Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx>;
}

#[cfg(feature = "workers")]
pub trait TCtxFunc<TCtx, TMarker>: Clone + Send + Sync + 'static
where
    TCtx: Send + 'static,
{
    // TODO: Support extracting `httpz::Request` and `httpz::Cookies`
    fn exec<'a>(&self) -> TCtxFuncResult<'a, TCtx>;
}

#[cfg(not(feature = "workers"))]
pub struct NoArgMarker(PhantomData<()>);

#[cfg(not(feature = "workers"))]
impl<TCtx, TFunc> TCtxFunc<TCtx, NoArgMarker> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce() -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'a>(&self, _request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        TCtxFuncResult::Value(Ok(self.clone()()))
    }
}

#[cfg(feature = "workers")]
pub struct NoArgMarker(PhantomData<()>);

#[cfg(feature = "workers")]
impl<TCtx, TFunc> TCtxFunc<TCtx, NoArgMarker> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce() -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'a>(&self) -> TCtxFuncResult<'a, TCtx> {
        TCtxFuncResult::Value(Ok(self.clone()()))
    }
}

#[cfg(not(feature = "workers"))]
pub struct OneArgAxumRequestMarker<T1>(PhantomData<T1>);

#[cfg(not(feature = "workers"))]
impl<T1, TCtx, TFunc> TCtxFunc<TCtx, OneArgAxumRequestMarker<T1>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1))
        }))
    }
}

#[cfg(not(feature = "workers"))]
pub struct TwoArgAxumRequestMarker<T1, T2>(PhantomData<(T1, T2)>);

#[cfg(not(feature = "workers"))]
impl<T1, T2, TCtx, TFunc> TCtxFunc<TCtx, TwoArgAxumRequestMarker<T1, T2>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2))
        }))
    }
}

#[cfg(not(feature = "workers"))]
pub struct ThreeArgAxumRequestMarker<T1, T2, T3>(PhantomData<(T1, T2, T3)>);

#[cfg(not(feature = "workers"))]
impl<T1, T2, T3, TCtx, TFunc> TCtxFunc<TCtx, ThreeArgAxumRequestMarker<T1, T2, T3>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2, T3) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
    <T3 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T3: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t3 = T3::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 3: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2, t3))
        }))
    }
}

#[cfg(not(feature = "workers"))]
pub struct FourArgAxumRequestMarker<T1, T2, T3, T4>(PhantomData<(T1, T2, T3, T4)>);

#[cfg(not(feature = "workers"))]
impl<T1, T2, T3, T4, TCtx, TFunc> TCtxFunc<TCtx, FourArgAxumRequestMarker<T1, T2, T3, T4>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2, T3, T4) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
    <T3 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T3: FromRequest<Vec<u8>> + Send + 'static,
    <T4 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T4: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t3 = T3::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 3: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t4 = T4::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 4: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2, t3, t4))
        }))
    }
}

#[cfg(not(feature = "workers"))]
pub struct FiveArgAxumRequestMarker<T1, T2, T3, T4, T5>(PhantomData<(T1, T2, T3, T4, T5)>);

#[cfg(not(feature = "workers"))]
impl<T1, T2, T3, T4, T5, TCtx, TFunc> TCtxFunc<TCtx, FiveArgAxumRequestMarker<T1, T2, T3, T4, T5>>
    for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2, T3, T4, T5) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
    <T3 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T3: FromRequest<Vec<u8>> + Send + 'static,
    <T4 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T4: FromRequest<Vec<u8>> + Send + 'static,
    <T5 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T5: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t3 = T3::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 3: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t4 = T4::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 4: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t5 = T5::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 5: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2, t3, t4, t5))
        }))
    }
}
