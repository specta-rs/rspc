use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin};

use super::{middleware::AlphaMiddlewareContext, MiddlewareArgMapper, MwV2, MwV2Result};

pub trait Ret: Debug + Send + 'static {}
impl<T: Debug + Send + 'static> Ret for T {}

pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

pub trait Executable<TRet: Ret, TFut: Fut<TRet>>: Send + 'static {
    type Fut: Fut<TRet>;

    fn exec(self) -> Self::Fut;
}
impl<TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + Send + 'static> Executable<TRet, TFut>
    for TFunc
{
    type Fut = TFut;

    fn exec(self) -> Self::Fut {
        self()
    }
}

pub struct Router<TCtx = (), TPlugin: Plugin = BasePlugin> {
    plugin: TPlugin,
    phantom: PhantomData<TCtx>,
}

impl<TCtx> Router<TCtx, BasePlugin> {
    pub fn new() -> Self {
        Self {
            plugin: BasePlugin {},
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TPlugin: Plugin> Router<TCtx, TPlugin>
where
    TCtx: Send + 'static,
{
    pub fn with<
        TMarker: Send + 'static,
        Mw: MwV2<TCtx, TMarker>
            + Fn(
                AlphaMiddlewareContext<
                    <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
                >,
                TCtx,
            ) -> Mw::Fut,
    >(
        self,
        mw: Mw,
    ) -> Router<TCtx, PluginJoiner<TPlugin, MapPlugin<TCtx, TMarker, Mw>>> {
        Router {
            plugin: PluginJoiner {
                a: self.plugin,
                b: MapPlugin(mw, PhantomData),
            },
            phantom: PhantomData,
        }
    }

    pub async fn query<TRet: Ret, TFut: Fut<TRet>>(self, func: impl Executable<TRet, TFut>) {
        let y = self.plugin.map(func);
        println!("\nBUILT\n");
        println!("{:?}\n", y.exec().await);
    }
}

pub trait Plugin {
    type Ret<TRet: Ret>: Ret;
    type Fut<TRet: Ret, TFut: Fut<TRet>>: Fut<Self::Ret<TRet>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>>: Executable<
        Self::Ret<TRet>,
        Self::Fut<TRet, TFut>,
    >;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>>(
        self,
        t: T,
    ) -> Self::Result<TRet, TFut, T>;
}

pub struct PluginJoiner<A: Plugin, B: Plugin> {
    a: A,
    b: B,
}

impl<A: Plugin, B: Plugin> Plugin for PluginJoiner<A, B> {
    type Ret<TRet: Ret> = A::Ret<B::Ret<TRet>>;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = A::Fut<B::Ret<TRet>, B::Fut<TRet, TFut>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>> =
        A::Result<B::Ret<TRet>, B::Fut<TRet, TFut>, B::Result<TRet, TFut, T>>;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>>(
        self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        self.a.map(self.b.map(t))
    }
}

pub struct BasePlugin;

impl Plugin for BasePlugin {
    type Ret<TRet: Ret> = TRet;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = TFut;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>> = T;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>>(
        self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        t
    }
}

pub struct MapPlugin<TCtx, TMarker, Mw>(Mw, PhantomData<(TCtx, TMarker)>)
where
    TCtx: 'static,
    TMarker: Send + 'static,
    Mw: MwV2<TCtx, TMarker>;

impl<TCtx, TMarker, Mw> Plugin for MapPlugin<TCtx, TMarker, Mw>
where
    TCtx: Send + 'static,
    TMarker: Send + 'static,
    Mw: MwV2<TCtx, TMarker>,
{
    type Ret<TRet: Ret> = TRet;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = Pin<Box<dyn Fut<Self::Ret<TRet>>>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>> =
        Demo<TCtx, Mw, TMarker, TRet>;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Executable<TRet, TFut>>(
        self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        Demo {
            mw: self.0,
            phantom: PhantomData,
        }
    }
}

pub struct Demo<TCtx, Mw, TMarker, TRet>
where
    TCtx: 'static,
    Mw: MwV2<TCtx, TMarker>,
    TMarker: Send + 'static,
    TRet: Ret,
{
    mw: Mw,
    phantom: PhantomData<(TCtx, TMarker, TRet)>,
}

impl<TCtx, Mw, TMarker, TRet> Executable<TRet, Pin<Box<dyn Fut<TRet>>>>
    for Demo<TCtx, Mw, TMarker, TRet>
where
    TCtx: Send + 'static,
    Mw: MwV2<TCtx, TMarker>,
    TMarker: Send + 'static,
    TRet: Ret,
{
    type Fut = Pin<Box<dyn Fut<TRet>>>;

    fn exec(self) -> Self::Fut {
        let y = self.mw.run_me();

        // TODO: Named future
        Box::pin(async move {
            let result = y.await;

            // let next_ctx = result.get_ctx(); // TODO

            // let data = t.exec().await; // TODO: Do this without cloning `t` by having two methods on `Executable`

            // println!("RESULT: {:?}", result);

            // let y = result.into_executable();

            // println!("MAP {} - AFTER", id);
            // data
            todo!();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mw2() {
        // TODO: Context switching

        let r = <Router>::new()
            .with(|mw, ctx| async move { mw.next(ctx) })
            .query(|| async move {
                println!("QUERY");
                "Query!".to_string()
            })
            .await;
    }
}
