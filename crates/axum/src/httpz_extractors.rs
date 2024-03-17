//! The future of this code in unsure. It will probs be removed or refactored once we support more than just Axum because all of the feature gating is bad.

use crate::{CookieJar, Request};
use rspc::ExecError;

use std::marker::PhantomData;

// TODO: Add an example to the repo of using the new cringe Axum extractors but put lots of warnings about how it's highly discouraged

pub trait TCtxFunc<TCtx, TMarker>: Clone + Send + Sync + 'static {
    fn exec<'req>(
        &self,
        req: httpz::Request,
        cookies: Option<CookieJar>,
    ) -> Result<TCtx, ExecError>
    where
        TCtx: Send + 'req;
}

pub struct NoArgMarker(PhantomData<()>);

impl<TCtx, TFunc> TCtxFunc<TCtx, NoArgMarker> for TFunc
where
    TFunc: FnOnce() -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'req>(
        &self,
        _req: httpz::Request,
        _cookies: Option<CookieJar>,
    ) -> Result<TCtx, ExecError>
    where
        TCtx: Send + 'req,
    {
        Ok(self.clone()()) // TODO: Avoiding clone here would be nice -> Why not use `Fn` instead of `FnOnce + Clone`?
    }
}

pub struct SingleArgMarker(PhantomData<()>);

impl<TCtx, TFunc> TCtxFunc<TCtx, SingleArgMarker> for TFunc
where
    TFunc: FnOnce(Request) -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'req>(&self, req: httpz::Request, cookies: Option<CookieJar>) -> Result<TCtx, ExecError>
    where
        TCtx: Send + 'req,
    {
        // TODO: Create `Option<CookieJar>` here and optionally return it so we only need to heap allocate it it's got a chance of being used
        // TODO: Avoiding clone here would be nice -> Why not use `Fn` instead of `FnOnce + Clone`?

        Ok(self.clone()(Request::new(req, cookies)))
    }
}
