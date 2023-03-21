use super::{Executable, Fut, Plugin, Ret};

pub struct PluginJoiner<A: Plugin, B: Plugin> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A: Plugin, B: Plugin> Plugin for PluginJoiner<A, B> {
    type Ret<TRet: Ret> = A::Ret<B::Ret<TRet>>;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = A::Fut<B::Ret<TRet>, B::Fut<TRet, TFut>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>> =
        A::Result<B::Ret<TRet>, B::Fut<TRet, TFut>, B::Result<TRet, TFut, T>>;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, Fut = TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        self.a.map(self.b.map(t))
    }
}
