use std::{
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use serde_json::Value;

use super::{middleware::AlphaMiddlewareContext, MiddlewareArgMapper, MwV2, MwV2Result};

pub trait Fut<TRet: Send + 'static>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Send + 'static, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

pub trait Executable<TBCtx, TFut>: Send + 'static
where
    TFut: Fut<Value>,
{
    type Fut: Fut<Value>;
    type Ctx;

    fn exec(self, ctx: Self::Ctx) -> Self::Fut;
}

pub struct Router<TBCtx = (), TPlugin: Plugin<TBCtx> = BasePlugin<TBCtx>> {
    plugin: TPlugin,
    phantom: PhantomData<TBCtx>,
}

impl<TBCtx> Router<TBCtx, BasePlugin<TBCtx>> {
    pub fn new() -> Self {
        Self {
            plugin: BasePlugin(PhantomData),
            phantom: PhantomData,
        }
    }
}

impl<TBCtx, TPlugin: Plugin<TBCtx>> Router<TBCtx, TPlugin>
where
    TBCtx: Send + 'static,
{
    pub fn with<
        TMarker: Send + 'static,
        Mw: MwV2<TBCtx, TMarker>
            + Fn(
                AlphaMiddlewareContext<
                    <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
                >,
                TBCtx,
            ) -> Mw::Fut,
    >(
        self,
        mw: Mw,
    ) -> Router<TBCtx, PluginJoiner<TBCtx, TPlugin, MapPlugin<TBCtx, TMarker, Mw>>> {
        Router {
            plugin: PluginJoiner {
                a: self.plugin,
                b: MapPlugin(mw, PhantomData),
                phantom: PhantomData,
            },
            phantom: PhantomData,
        }
    }

    pub async fn query<
        TRet: Debug + Send + 'static,
        TFut: Fut<TRet>,
        TFunc: Fn() -> TFut + Send + 'static,
    >(
        self,
        func: TFunc,
        ctx: TBCtx,
    ) {
        let y = ResolverPluginExecutable(func, PhantomData);
        let y = self.plugin.map(y);
        println!("\nBUILT\n");
        // println!("{:?}\n", y.exec(ctx).await); // TODO
    }
}

pub trait Plugin<TBCtx> {
    // TODO: Maybe remove `Fut` in favor of `Result::Output` or whatever????
    type Fut<TFut: Fut<Value>>: Fut<Value>;
    type Result<TFut: Fut<Value>, T: Executable<TBCtx, TFut>>: Executable<TBCtx, Self::Fut<TFut>>;

    fn map<TFut: Fut<Value>, T: Executable<TBCtx, TFut>>(self, t: T) -> Self::Result<TFut, T>;
}

pub struct PluginJoiner<TBCtx, A, B>
where
    TBCtx: Send + 'static,
    A: Plugin<TBCtx>,
    B: Plugin<TBCtx>,
{
    a: A,
    b: B,
    phantom: PhantomData<TBCtx>,
}

impl<TBCtx, A, B> Plugin<TBCtx> for PluginJoiner<TBCtx, A, B>
where
    TBCtx: Send + 'static,
    A: Plugin<TBCtx>,
    B: Plugin<TBCtx>,
{
    type Fut<TFut: Fut<Value>> = A::Fut<B::Fut<TFut>>;
    type Result<TFut: Fut<Value>, T: Executable<TBCtx, TFut>> =
        A::Result<B::Fut<TFut>, B::Result<TFut, T>>;

    fn map<TFut: Fut<Value>, T: Executable<TBCtx, TFut>>(self, t: T) -> Self::Result<TFut, T> {
        self.a.map(self.b.map(t))
    }
}

pub struct ResolverPluginExecutable<TBCtx, TRet, TFut, TFunc>(TFunc, PhantomData<(TBCtx, TRet)>)
where
    TBCtx: Send + 'static,
    TRet: Debug + Send + 'static,
    TFut: Fut<TRet>,
    TFunc: Fn() -> TFut + Send + 'static;

impl<TBCtx, TRet, TFut, TFunc> Executable<TBCtx, ResolverPluginFut<TRet, TFut>>
    for ResolverPluginExecutable<TBCtx, TRet, TFut, TFunc>
where
    TBCtx: Send + 'static,
    TRet: Debug + Send + 'static,
    TFut: Fut<TRet>,
    TFunc: Fn() -> TFut + Send + 'static,
{
    type Fut = ResolverPluginFut<TRet, TFut>;
    type Ctx = (); // TODO

    fn exec(self, _ctx: Self::Ctx) -> Self::Fut {
        ResolverPluginFut((self.0)(), PhantomData)
    }
}

pub struct ResolverPluginFut<TRet, TFut>(TFut, PhantomData<TRet>)
where
    TRet: Debug + Send + 'static,
    TFut: Fut<TRet>;

impl<TRet, TFut> Future for ResolverPluginFut<TRet, TFut>
where
    TRet: Debug + Send + 'static,
    TFut: Fut<TRet>,
{
    type Output = Value;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

pub struct BasePlugin<TBCtx>(PhantomData<TBCtx>);

impl<TBCtx> Plugin<TBCtx> for BasePlugin<TBCtx> {
    type Fut<TFut: Fut<Value>> = TFut;
    type Result<TFut: Fut<Value>, T: Executable<TBCtx, TFut>> = T;

    fn map<TFut: Fut<Value>, T: Executable<TBCtx, TFut>>(self, t: T) -> Self::Result<TFut, T> {
        t
    }
}

pub struct MapPlugin<TBCtx, TMarker, Mw>(Mw, PhantomData<(TBCtx, TMarker)>)
where
    TBCtx: 'static,
    TMarker: Send + 'static,
    Mw: MwV2<TBCtx, TMarker>;

impl<TBCtx, TMarker, Mw> Plugin<TBCtx> for MapPlugin<TBCtx, TMarker, Mw>
where
    TBCtx: Send + 'static,
    TMarker: Send + 'static,
    Mw: MwV2<TBCtx, TMarker>,
{
    type Fut<TFut: Fut<Value>> = Pin<Box<dyn Fut<Value>>>;
    type Result<TFut: Fut<Value>, T: Executable<TBCtx, TFut>> = Demo<TBCtx, Mw, TMarker>;

    fn map<TFut: Fut<Value>, T: Executable<TBCtx, TFut>>(self, t: T) -> Self::Result<TFut, T> {
        Demo {
            mw: self.0,
            phantom: PhantomData,
        }
    }
}

pub struct Demo<TBCtx, Mw, TMarker>
where
    TBCtx: 'static,
    Mw: MwV2<TBCtx, TMarker>,
    TMarker: Send + 'static,
{
    mw: Mw,
    phantom: PhantomData<(TBCtx, TMarker)>,
}

impl<TBCtx, Mw, TMarker> Executable<TBCtx, Pin<Box<dyn Fut<Value>>>> for Demo<TBCtx, Mw, TMarker>
where
    TBCtx: Send + 'static,
    Mw: MwV2<TBCtx, TMarker>,
    TMarker: Send + 'static,
{
    type Fut = Pin<Box<dyn Fut<Value>>>;
    type Ctx = Mw::NewCtx;
    // type State = AlphaMiddlewareContext<
    //     <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
    // >;

    fn exec(self, _ctx: Self::Ctx) -> Self::Fut {
        // let state = self.mw.get_state();

        // let y = self.mw.run_me();

        // TODO: Named future
        Box::pin(async move {
            // let result = y.await;

            // let next_ctx = result.get_ctx(); // TODO

            // let data = t.exec().await; // TODO: Do this without cloning `t` by having two methods on `Executable`

            // println!("RESULT: {:?}", result);

            // let y = result.into_executable();

            // println!("MAP {} - AFTER", id);
            // data

            Value::Null // TODO
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
            // TODO: Take ctx and arg as params
            .query(
                || async move {
                    println!("QUERY");
                    "Query!".to_string()
                },
                (),
            )
            .await;
    }
}
