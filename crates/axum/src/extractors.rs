use axum::{extract::FromRequestParts, http::request::Parts};
use futures::executor::block_on;
use rspc::ExecError;

use std::marker::PhantomData;

pub trait TCtxFunc<TCtx, TMarker>: Clone + Send + Sync + 'static {
    fn exec<'req>(&self, parts: Parts) -> Result<TCtx, ExecError>
    where
        TCtx: Send + 'req;
}

pub struct NoArgMarker(PhantomData<()>);

impl<TCtx, TFunc> TCtxFunc<TCtx, NoArgMarker> for TFunc
where
    TFunc: FnOnce() -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'req>(&self, _parts: Parts) -> Result<TCtx, ExecError>
    where
        TCtx: Send + 'req,
    {
        Ok(self.clone()()) // TODO: Avoiding clone here would be nice -> Why not use `Fn` instead of `FnOnce + Clone`?
    }
}

pub struct SingleArgMarker<E1>(PhantomData<E1>);

impl<TCtx, TFunc, E1: FromRequestParts<()>> TCtxFunc<TCtx, SingleArgMarker<E1>> for TFunc
where
    TFunc: FnOnce(E1) -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'req>(&self, mut parts: Parts) -> Result<TCtx, ExecError>
    where
        TCtx: Send + 'req,
    {
        // TODO: Don't `block_on` here.
        let req = block_on(E1::from_request_parts(&mut parts, &()))
            // TODO: Preserve actual error
            .map_err(|_| ExecError::AxumExtractorError)?;
        // TODO: Avoiding clone here would be nice -> Why not use `Fn` instead of `FnOnce + Clone`?
        Ok(self.clone()(req))
    }
}
