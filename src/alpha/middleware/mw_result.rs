use std::{
    fmt::Debug,
    future::{Future, Ready},
    marker::PhantomData,
};

use serde_json::Value;

use crate::{alpha::MiddlewareArgMapper, internal::RequestContext};

pub trait Ret: Debug + Send + Sync + 'static {}
impl<T: Debug + Send + Sync + 'static> Ret for T {}

pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

pub trait Func<TRet: Ret, TFut: Fut<TRet>>: Fn() -> TFut + Send + Sync + 'static {}
impl<TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + Send + Sync + 'static> Func<TRet, TFut>
    for TFunc
{
}

pub trait Executable2: Send + Sync + 'static {
    type Fut: Future<Output = Value> + Send;

    fn call(self, v: Value) -> Self::Fut;
}

impl<TFut: Fut<Value>, TFunc: FnOnce(Value) -> TFut + Send + Sync + 'static> Executable2 for TFunc {
    type Fut = TFut;

    fn call(self, v: Value) -> Self::Fut {
        (self)(v)
    }
}

pub struct Executable2Placeholder {}

impl Executable2 for Executable2Placeholder {
    type Fut = Ready<Value>;

    fn call(self, _: Value) -> Self::Fut {
        unreachable!();
    }
}

pub trait MwV2Result {
    type Ctx;
    type MwMapper: MiddlewareArgMapper;
    type Resp: Executable2;

    fn explode(self) -> (Self::Ctx, Value, RequestContext, Option<Self::Resp>);
}

pub struct MwResultWithCtx<TLCtx, M, TResp>
where
    M: MiddlewareArgMapper,
    TResp: Executable2,
{
    pub(crate) input: Value,
    pub(crate) req: RequestContext,
    pub(crate) ctx: Option<TLCtx>,
    pub(crate) resp: Option<TResp>,
    pub(crate) phantom: PhantomData<M>,
}

impl<TLCtx, M, TResp> MwResultWithCtx<TLCtx, M, TResp>
where
    M: MiddlewareArgMapper,
    TResp: Executable2,
{
    pub fn resp<E: Executable2>(self, handler: E) -> MwResultWithCtx<TLCtx, M, E> {
        MwResultWithCtx {
            input: self.input,
            req: self.req,
            ctx: self.ctx,
            resp: Some(handler),
            phantom: PhantomData,
        }
    }
}

impl<TLCtx, M, TResp> MwV2Result for MwResultWithCtx<TLCtx, M, TResp>
where
    M: MiddlewareArgMapper,
    TResp: Executable2,
{
    type Ctx = TLCtx;
    type MwMapper = M;
    type Resp = TResp;

    fn explode(self) -> (Self::Ctx, Value, RequestContext, Option<Self::Resp>) {
        (self.ctx.unwrap(), self.input, self.req, self.resp)
    }
}

// TODO: Fix this
// #[cfg(test)]
// mod tests {
//     use crate::alpha::middleware::AlphaMiddlewareContext;

//     use super::*;

//     fn mw<TMarker, Mw>(m: Mw)
//     where
//         TMarker: Send + 'static,
//         Mw: MwV2<(), TMarker>
//             + Fn(
//                 AlphaMiddlewareContext<
//                     <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
//                 >,
//                 (),
//             ) -> Mw::Fut,
//     {
//     }

//     #[tokio::test]
//     async fn test_mw_results() {
//         // Pass through ctx
//         mw(|mw, ctx| async move { mw.next(ctx) });

//         // Switch ctx
//         mw(|mw, ctx| async move { mw.next(()) });

//         // Handle response
//         mw(|mw, ctx| async move { mw.next(()).resp(|result| async move { result }) });

//         // Middleware args
//         mw(|mw, ctx| async move {
//             let my_mappers_state = mw.state;
//             mw.args::<()>().next(())
//         });

//         // TODO: Handle response returning Result
//         // mw(|mw, ctx| async move { mw.next(()).resp(|result| async move { Ok(result) }) });

//         // TODO: Handle only query/mutation response
//         // mw(|mw, ctx| async move {
//         //     mw.args::<()>().next(()).raw_resp(|resp| {
//         //         match resp {
//         //             ValueOrStream::Value(_) => {},
//         //             ValueOrStream::Stream(_) => {},
//         //         }
//         //     })
//         // });

//         // TODO: Replace stream
//         // mw(|mw, ctx| async move {
//         //     mw.args::<()>().next(()).stream(|stream| {
//         //         async_stream::stream! {
//         //             while let Some(msg) = stream.next().await {
//         //                 yield msg;
//         //             }
//         //         }
//         //     })
//         // });
//     }
// }
