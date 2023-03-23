use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use std::future::Future;

use super::{AlphaMiddlewareContext, Executable, Fut, MiddlewareResult, Plugin, Ret};

// pub trait AlphaMw<TLCtx> {
//     type Fut: Future<Output = Self::Ret>;
//     type Ret: MiddlewareResult;
// }

// impl<TLCtx, F, Fu, R> AlphaMw<TLCtx> for F
// where
//     F: Fn(AlphaMiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
//     Fu: Future<Output = R> + Send + 'static,
//     R: MiddlewareResult,
// {
//     type Fut = Fu;
//     type Ret = R;
// }

// // TODO: Rename all these types

// pub struct MapPlugin<TLCtx, M>(pub(crate) M, pub(crate) PhantomData<TLCtx>)
// where
//     M: AlphaMw<TLCtx>;

// impl<TLCtx, M> Plugin for MapPlugin<TLCtx, M>
// where
//     M: AlphaMw<TLCtx>,
// {
//     type Ret<TRet: Ret> = TRet;
//     type Fut<TRet: Ret, TFut: Fut<TRet>> = MapPluginFuture<Self::Ret<TRet>, TFut>;
//     type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>> =
//         MapPluginResult<Self::Ret<TRet>, TFut, T>;

//     fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>>(
//         &self,
//         t: T,
//     ) -> Self::Result<TRet, TFut, T> {
//         println!("BUILD");
//         // MapPluginResult(t, PhantomData)
//         todo!();
//     }
// }

// pub struct MapPluginResult<TRet, TFut, T>(T, PhantomData<(TRet, TFut)>);

// impl<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>> Executable<TRet>
//     for MapPluginResult<TRet, TFut, T>
// {
//     type Fut = MapPluginFuture<TRet, TFut>;

//     fn call(&self) -> Self::Fut {
//         println!("MAP - BEFORE");

//         MapPluginFuture {
//             fut: self.0.call(),
//             phantom: PhantomData,
//         }
//     }
// }

// pin_project! {
//     pub struct MapPluginFuture<TRet: Ret, TFut: Fut<TRet>> {
//         #[pin]
//         fut: TFut,
//         // fut2: THandler,
//         phantom: PhantomData<TRet>
//     }
// }

// impl<TRet: Ret, TFut: Fut<TRet>> Future for MapPluginFuture<TRet, TFut> {
//     type Output = TRet;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();
//         match this.fut.poll(cx) {
//             Poll::Ready(data) => {
//                 println!("MAP - AFTER");
//                 Poll::Ready(data)
//             }
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }
