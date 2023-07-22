use std::{
    convert::Infallible,
    fmt::Debug,
    future::{Future, Ready},
};

use bytes::Bytes;
use serde_json::Value;

use crate::{
    internal::{middleware::RequestContext, DynBody},
    ExecError,
};

mod private {
    // TODO
}

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
    type Ctx: Send + Sync + 'static;
    type Resp: Executable2;

    type Body: http_body::Body<Data = Bytes, Error = Infallible> + Send + Sync;

    // TODO: Seal this and make it private
    fn explode(
        self,
    ) -> Result<
        (
            Self::Ctx,
            Value,
            RequestContext,
            Option<Self::Resp>,
            Self::Body,
        ),
        ExecError,
    >;
}

// pub trait MwV2Result2<'a> {
//     type Body: http_body::Body<Data = Bytes, Error = Infallible> + Send + Sync;

//     // TODO: Seal this and make it private
//     fn explode(
//         self,
//     ) -> Result<
//         (
//             Self::Ctx,
//             Value,
//             RequestContext,
//             Option<Self::Resp>,
//             Self::Body,
//         ),
//         ExecError,
//     >;
// }

// TODO: Seal this and rename it
pub struct MwResultWithCtx<'a, TLCtx, TResp> {
    pub(crate) input: Value,
    pub(crate) req: RequestContext,
    pub(crate) ctx: TLCtx,
    pub(crate) body: &'a mut DynBody,
    pub(crate) resp: Option<TResp>,
}

impl<'a, TLCtx, TResp: Executable2> MwResultWithCtx<'a, TLCtx, TResp> {
    pub fn map<E: Executable2>(self, handler: E) -> MwResultWithCtx<'a, TLCtx, E> {
        MwResultWithCtx {
            input: self.input,
            req: self.req,
            ctx: self.ctx,
            body: self.body,
            resp: Some(handler),
        }
    }
}

impl<'a, TLCtx, TResp> MwV2Result for MwResultWithCtx<'a, TLCtx, TResp>
where
    TLCtx: Send + Sync + 'static,
    TResp: Executable2,
{
    type Ctx = TLCtx;
    type Resp = TResp;

    type Body = &'a mut DynBody;

    fn explode(
        self,
    ) -> Result<
        (
            Self::Ctx,
            Value,
            RequestContext,
            Option<Self::Resp>,
            Self::Body,
        ),
        ExecError,
    > {
        Ok((self.ctx, self.input, self.req, self.resp, self.body))
    }
}

impl<'a, TLCtx, TResp> MwV2Result for Result<MwResultWithCtx<'a, TLCtx, TResp>, crate::Error>
where
    TLCtx: Send + Sync + 'static,
    TResp: Executable2,
{
    type Ctx = TLCtx;
    type Resp = TResp;

    type Body = &'a mut DynBody;

    fn explode(
        self,
    ) -> Result<
        (
            Self::Ctx,
            Value,
            RequestContext,
            Option<Self::Resp>,
            Self::Body,
        ),
        ExecError,
    > {
        match self {
            Ok(mw_result) => Ok(mw_result.explode()?),
            Err(err) => Err(err.into()),
        }
    }
}
