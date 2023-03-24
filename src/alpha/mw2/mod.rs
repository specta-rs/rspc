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

pub trait Executable<TFut>: Send + 'static
where
    TFut: Fut<Value>,
{
    type Fut: Fut<Value>;
    type Ctx;
    type State; // TODO: Type erase this into the trait

    // TODO: `state` shouldn't be an arg -> It should be handled internally
    fn exec(self, ctx: Self::Ctx, state: Self::State) -> Self::Fut;
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

    pub async fn query<
        TRet: Debug + Send + 'static,
        TFut: Fut<TRet>,
        TFunc: Fn() -> TFut + Send + 'static,
    >(
        self,
        func: TFunc,
    ) {
        let y = ResolverPluginExecutable(func, PhantomData);
        let y = self.plugin.map(y);
        println!("\nBUILT\n");
        // println!("{:?}\n", y.exec((), ()).await); // TODO
    }
}

pub trait Plugin {
    // TODO: Maybe remove `Fut` in favor of `Result::Output` or whatever????
    type Fut<TFut: Fut<Value>>: Fut<Value>;
    type Result<TFut: Fut<Value>, T: Executable<TFut>>: Executable<Self::Fut<TFut>>;

    fn map<TFut: Fut<Value>, T: Executable<TFut>>(self, t: T) -> Self::Result<TFut, T>;
}

pub struct PluginJoiner<A: Plugin, B: Plugin> {
    a: A,
    b: B,
}

impl<A: Plugin, B: Plugin> Plugin for PluginJoiner<A, B> {
    type Fut<TFut: Fut<Value>> = A::Fut<B::Fut<TFut>>;
    type Result<TFut: Fut<Value>, T: Executable<TFut>> =
        A::Result<B::Fut<TFut>, B::Result<TFut, T>>;

    fn map<TFut: Fut<Value>, T: Executable<TFut>>(self, t: T) -> Self::Result<TFut, T> {
        self.a.map(self.b.map(t))
    }
}

pub struct ResolverPluginExecutable<TRet, TFut, TFunc>(TFunc, PhantomData<TRet>)
where
    TRet: Debug + Send + 'static,
    TFut: Fut<TRet>,
    TFunc: Fn() -> TFut + Send + 'static;

impl<TRet, TFut, TFunc> Executable<ResolverPluginFut<TRet, TFut>>
    for ResolverPluginExecutable<TRet, TFut, TFunc>
where
    TRet: Debug + Send + 'static,
    TFut: Fut<TRet>,
    TFunc: Fn() -> TFut + Send + 'static,
{
    type Fut = ResolverPluginFut<TRet, TFut>;
    type Ctx = ();
    type State = ();

    fn exec(self, _ctx: Self::State, _state: Self::State) -> Self::Fut {
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

pub struct BasePlugin;

impl Plugin for BasePlugin {
    type Fut<TFut: Fut<Value>> = TFut;
    type Result<TFut: Fut<Value>, T: Executable<TFut>> = T;

    fn map<TFut: Fut<Value>, T: Executable<TFut>>(self, t: T) -> Self::Result<TFut, T> {
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
    type Fut<TFut: Fut<Value>> = Pin<Box<dyn Fut<Value>>>;
    type Result<TFut: Fut<Value>, T: Executable<TFut>> = Demo<TCtx, Mw, TMarker>;

    fn map<TFut: Fut<Value>, T: Executable<TFut>>(self, t: T) -> Self::Result<TFut, T> {
        Demo {
            mw: self.0,
            phantom: PhantomData,
        }
    }
}

pub struct Demo<TCtx, Mw, TMarker>
where
    TCtx: 'static,
    Mw: MwV2<TCtx, TMarker>,
    TMarker: Send + 'static,
{
    mw: Mw,
    phantom: PhantomData<(TCtx, TMarker)>,
}

impl<TCtx, Mw, TMarker> Executable<Pin<Box<dyn Fut<Value>>>> for Demo<TCtx, Mw, TMarker>
where
    TCtx: Send + 'static,
    Mw: MwV2<TCtx, TMarker>,
    TMarker: Send + 'static,
{
    type Fut = Pin<Box<dyn Fut<Value>>>;
    type Ctx = Mw::NewCtx;
    type State = AlphaMiddlewareContext<
        <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
    >;

    fn exec(self, _ctx: Self::Ctx, _state: Self::State) -> Self::Fut {
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
            .query(|| async move {
                println!("QUERY");
                "Query!".to_string()
            })
            .await;
    }
}
