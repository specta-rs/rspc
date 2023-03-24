use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin};

use super::{middleware::AlphaMiddlewareContext, MiddlewareArgMapper, MwV2, MwV2Result};

pub trait Ret: Debug + 'static {}
impl<T: Debug + 'static> Ret for T {}

pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

pub trait Func<TRet: Ret, TFut: Fut<TRet>>: FnOnce() -> TFut + Send + 'static {}
impl<TRet: Ret, TFut: Fut<TRet>, TFunc: FnOnce() -> TFut + Send + 'static> Func<TRet, TFut>
    for TFunc
{
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

impl<TCtx, TPlugin: Plugin> Router<TCtx, TPlugin> {
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
    ) -> Router<TCtx, PluginJoiner<TPlugin, MapPlugin>> {
        Router {
            plugin: PluginJoiner {
                a: self.plugin,
                b: MapPlugin("A".into()),
            },
            phantom: PhantomData,
        }
    }

    pub async fn query<TRet: Ret, TFut: Fut<TRet>>(&self, func: impl Func<TRet, TFut>) {
        let y = self.plugin.map(func);
        println!("\nBUILT\n");
        println!("{:?}\n", y().await);
    }
}

pub trait Plugin {
    type Ret<TRet: Ret>: Ret;
    type Fut<TRet: Ret, TFut: Fut<TRet>>: Fut<Self::Ret<TRet>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>>: Func<
        Self::Ret<TRet>,
        Self::Fut<TRet, TFut>,
    >;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>>(
        &self,
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
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>> =
        A::Result<B::Ret<TRet>, B::Fut<TRet, TFut>, B::Result<TRet, TFut, T>>;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        self.a.map(self.b.map(t))
    }
}

pub struct BasePlugin;

impl Plugin for BasePlugin {
    type Ret<TRet: Ret> = TRet;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = TFut;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>> = T;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        println!("BUILD BASE");
        t
    }
}

pub struct MapPlugin(String);

impl Plugin for MapPlugin {
    type Ret<TRet: Ret> = TRet;
    type Fut<TRet: Ret, TFut: Fut<TRet>> = Pin<Box<dyn Fut<Self::Ret<TRet>>>>;
    type Result<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>> =
        Box<dyn Func<Self::Ret<TRet>, Self::Fut<TRet, TFut>>>;

    fn map<TRet: Ret, TFut: Fut<TRet>, T: Func<TRet, TFut>>(
        &self,
        t: T,
    ) -> Self::Result<TRet, TFut, T> {
        let id = self.0.clone();
        println!("BUILD {}", id);
        Box::new(move || {
            Box::pin(async move {
                println!("MAP {} - BEFORE", id);
                let data = t().await;
                println!("MAP {} - AFTER", id);
                data
            })
        })
    }
}

async fn todo() {
    let r = <Router>::new()
        .with(|mw, ctx| async move { mw.next(ctx) })
        .query(|| async move {
            println!("QUERY");
            "Query!".to_string()
        })
        .await;
}
