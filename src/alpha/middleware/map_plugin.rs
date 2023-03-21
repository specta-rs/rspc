use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use std::future::Future;

use super::{Executable, Fut, Plugin, Ret};

pub struct MapPlugin(pub(crate) String);

impl Plugin for MapPlugin {
    type Ret<TRet: Ret> = TRet;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = MapPluginFuture<Self::Ret<TRet>, TFut>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>> =
        MapPluginResult<Self::Ret<TRet>, TFut, T>;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        let id = self.0.clone();
        println!("BUILD {}", id);
        MapPluginResult(t, PhantomData)
    }
}

pub struct MapPluginResult<TRet, TFut, T>(T, PhantomData<(TRet, TFut)>);

impl<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>> Executable<TRet>
    for MapPluginResult<TRet, TFut, T>
{
    type Fut = MapPluginFuture<TRet, TFut>;

    fn call(&self) -> Self::Fut {
        println!("MAP - BEFORE");

        MapPluginFuture {
            fut: self.0.call(),
            phantom: PhantomData,
        }
    }
}

pin_project! {
    pub struct MapPluginFuture<TRet: Ret, TFut: Fut<TRet>> {
        #[pin]
        fut: TFut,
        phantom: PhantomData<TRet>
    }
}

impl<TRet: Ret, TFut: Fut<TRet>> Future for MapPluginFuture<TRet, TFut> {
    type Output = TRet;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Ready(data) => {
                println!("MAP - AFTER");
                Poll::Ready(data)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
