use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{Error, IntoResolverResult, ResolverResult};

pub trait Resolver<TCtx, TResult, TMarker> {
    fn exec(&self, ctx: TCtx, arg: Value) -> Result<ResolverResult, Error>;
}

pub struct NoArgMarker<TResultMarker>(/* private */ PhantomData<TResultMarker>);
impl<TFunc, TCtx, TResult, TResultMarker> Resolver<TCtx, TResult, NoArgMarker<TResultMarker>>
    for TFunc
where
    TFunc: Fn() -> TResult,
    TResult: IntoResolverResult<TResultMarker>,
{
    fn exec(&self, _ctx: TCtx, _arg: Value) -> Result<ResolverResult, Error> {
        Ok(self().into_resolver_result())
    }
}

pub struct SingleArgMarker<TResultMarker>(/* private */ PhantomData<TResultMarker>);
impl<TFunc, TCtx, TResult, TResultMarker> Resolver<TCtx, TResult, SingleArgMarker<TResultMarker>>
    for TFunc
where
    TFunc: Fn(TCtx) -> TResult,
    TResult: IntoResolverResult<TResultMarker>,
{
    fn exec(&self, ctx: TCtx, _arg: Value) -> Result<ResolverResult, Error> {
        Ok(self(ctx).into_resolver_result())
    }
}

pub struct DoubleArgMarker<TArg, TResultMarker>(
    /* private */ PhantomData<(TArg, TResultMarker)>,
);
impl<TFunc, TCtx, TArg, TResult, TResultMarker>
    Resolver<TCtx, TResult, DoubleArgMarker<TArg, TResultMarker>> for TFunc
where
    TArg: DeserializeOwned,
    TFunc: Fn(TCtx, TArg) -> TResult,
    TResult: IntoResolverResult<TResultMarker>,
{
    fn exec(&self, ctx: TCtx, arg: Value) -> Result<ResolverResult, Error> {
        let arg = serde_json::from_value(arg).map_err(|err| Error::ErrDeserializingArg(err))?;
        Ok(self(ctx, arg).into_resolver_result())
    }
}
