use std::{
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

mod base_plugin;
mod executable;
mod map_plugin;
mod plugin;
mod plugin_joiner;

pub use base_plugin::*;
pub use executable::*;
pub use map_plugin::*;
pub use plugin::*;
pub use plugin_joiner::*;

pub trait Ret: Debug + Send + Sync + 'static {}
impl<T: Debug + Send + Sync + 'static> Ret for T {}

pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

pub trait Func<TRet: Ret, TFut: Fut<TRet>>: Fn() -> TFut + Send + Sync + 'static {}
impl<TRet: Ret, TFut: Fut<TRet>, TFunc: Fn() -> TFut + Send + Sync + 'static> Func<TRet, TFut>
    for TFunc
{
}

// TODO: Merge this into other stuff
pub struct Router<TPlugin: Plugin = BasePlugin> {
    plugin: TPlugin,
}

impl Router<BasePlugin> {
    pub fn new() -> Self {
        Self {
            plugin: BasePlugin {},
        }
    }
}

impl<TPlugin: Plugin> Router<TPlugin> {
    pub fn plugin<T: Plugin>(self, t: T) -> Router<PluginJoiner<TPlugin, T>> {
        Router {
            plugin: PluginJoiner {
                a: self.plugin,
                b: t,
            },
        }
    }

    pub async fn query<TRet: Ret, TFut: Fut<TRet>>(&self, func: impl Func<TRet, TFut>) {
        let y = self.plugin.map(func);
        println!("\nBUILT\n");
        println!("{:?}\n", y.call().await);
    }
}

#[tokio::main]
async fn main() {
    use map_plugin::MapPlugin;

    let r = <Router>::new()
        .plugin(MapPlugin("A".into()))
        .plugin(MapPlugin("B".into()))
        .query(|| async move {
            println!("QUERY");
            "Query!".to_string()
        })
        .await;

    // let r = <Router>::new()
    //     .plugin(MapPlugin {})
    //     .query(|| "Query!".to_string());

    // let r = <Router>::new()
    //     .plugin(MapPlugin {})
    //     .plugin(OverridePlugin {})
    //     .query(|| "Query!".to_string());
}
