use std::{
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

mod executable;
mod mw;
mod mw_result;

pub use executable::*;
pub use mw::*;
pub use mw_result::*;

use crate::middleware;

#[deprecated = "Cringe type alert"]
pub trait Ret: Debug + Send + Sync + 'static {}
impl<T: Debug + Send + Sync + 'static> Ret for T {}

#[deprecated = "Cringe type alert"]
pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

#[deprecated = "Cringe type alert"]
pub trait Func<TRet: Ret, TFut: Fut<TRet>>: Fn() -> TFut + Send + Sync + 'static {}
impl<TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + Send + Sync + 'static> Func<TRet, TFut>
    for TFunc
{
}

#[deprecated = "Cringe type alert"]
pub trait MiddlewareResult {}
