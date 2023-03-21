//! TODO: Rename this from `Plugin` to `Middleware` or something

use super::{Executable, Fut, Ret};

pub trait Plugin {
    type Ret<TRet: Ret>: Ret;
    type Fut<TRet: Ret, TFut: Fut<TRet>>: Fut<Self::Ret<TRet>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>>: Executable<
        Self::Ret<TRet>,
        Fut = Self::Fut<TRet, TFut>,
    >;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T>;
}
