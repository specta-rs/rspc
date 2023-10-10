use std::{
    fmt::Debug,
    future::{Future, Ready},
    marker::PhantomData,
};

use serde_json::Value;

use super::RequestContext;
use crate::error::{ExecError, IntoResolverError};

pub trait Ret: Debug + Send + Sync + 'static {}
impl<T: Debug + Send + Sync + 'static> Ret for T {}

pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

// TODO: Remove this if possible it's something to do with optional function callback but can we cheat it with two different impls for `.call` based on generic?
pub trait Executable2: Send + Sync + 'static {
    type Fut: Future<Output = Value> + Send;

    fn call(&self, v: Value) -> Self::Fut;
}

impl<TFut: Fut<Value>, TFunc: Fn(Value) -> TFut + Send + Sync + 'static> Executable2 for TFunc {
    type Fut = TFut;

    fn call(&self, v: Value) -> Self::Fut {
        (self)(v)
    }
}

pub struct Executable2Placeholder {}

impl Executable2 for Executable2Placeholder {
    type Fut = Ready<Value>;

    fn call(&self, _: Value) -> Self::Fut {
        unreachable!();
    }
}

// #[deprecated = "TODO: We probs have to remove this. Sadge!"] // TODO: Deal with this type and seal it
pub trait MwV2Result {
    type Resp: Executable2;

    // TODO: Seal this and make it private
    fn explode(
        self,
    ) -> Result<
        (
            // TODO: Remove this first arg
            (),
            Value,
            RequestContext,
            Option<Self::Resp>,
        ),
        ExecError,
    >;
}

// TODO: Seal this and rename it
pub struct MwResultWithCtx<TResp> {
    pub(crate) input: Value,
    pub(crate) req: RequestContext,
    pub(crate) resp: Option<TResp>,
}

impl<TResp: Executable2> MwResultWithCtx<TResp> {
    pub fn map<E: Executable2>(self, handler: E) -> MwResultWithCtx<E> {
        MwResultWithCtx {
            input: self.input,
            req: self.req,
            resp: Some(handler),
        }
    }
}

impl<TResp> MwV2Result for MwResultWithCtx<TResp>
where
    TResp: Executable2,
{
    type Resp = TResp;

    fn explode(
        self,
    ) -> Result<
        (
            (), /* Self::Ctx */
            Value,
            RequestContext,
            Option<Self::Resp>,
        ),
        ExecError,
    > {
        Ok(((), self.input, self.req, self.resp))
    }
}

impl<TResp, TError> MwV2Result for Result<MwResultWithCtx<TResp>, TError>
where
    TResp: Executable2,
    TError: IntoResolverError,
{
    type Resp = TResp;

    fn explode(
        self,
    ) -> Result<
        (
            (), /* Self::Ctx */
            Value,
            RequestContext,
            Option<Self::Resp>,
        ),
        ExecError,
    > {
        self.map_err(|e| e.into_resolver_error().into())
            .and_then(|r| r.explode())
    }
}
