use std::future::Future;

use super::{Fut, Ret};

pub trait Executable<Result> {
    type Fut: Future<Output = Result>;

    fn call(&self) -> Self::Fut;
}

impl<TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + 'static> Executable<TRet> for TFunc {
    type Fut = TFut;

    fn call(&self) -> Self::Fut {
        (self)()
    }
}
