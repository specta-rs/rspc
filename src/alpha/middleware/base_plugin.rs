use super::{Executable, Fut, Plugin, Ret};

pub struct BasePlugin;

impl Plugin for BasePlugin {
    type Ret<TRet: Ret> = TRet;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = TFut;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>> = T;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        println!("BUILD BASE");
        t
    }
}
