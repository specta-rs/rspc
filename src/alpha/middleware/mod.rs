// TODO: Refactor type names though this whole package cause it's currently pretty messy
#![allow(deprecated)] // TODO: Remove once stuff is stablised

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
mod mw;
mod mw_builder;
mod mw_result;
mod plugin;
mod plugin_joiner;

pub use base_plugin::*;
pub use executable::*;
pub use map_plugin::*;
pub use mw::*;
pub use mw_builder::*;
pub use mw_result::*;
pub use plugin::*;
pub use plugin_joiner::*;

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

// #[deprecated = "Cringe type alert"]
// pub struct PluginRouter<TLCtx = (), TPlugin: Plugin = BasePlugin> {
//     plugin: TPlugin,
//     phantom: PhantomData<TLCtx>,
// }

// impl<TLCtx> PluginRouter<TLCtx, BasePlugin> {
//     pub fn new() -> Self {
//         Self {
//             plugin: BasePlugin {},
//             phantom: PhantomData,
//         }
//     }
// }

// impl<TLCtx, TPlugin: Plugin> PluginRouter<TLCtx, TPlugin> {
//     pub fn plugin<T: Plugin>(self, t: T) -> PluginRouter<TLCtx, PluginJoiner<TPlugin, T>> {
//         PluginRouter {
//             plugin: PluginJoiner {
//                 a: self.plugin,
//                 b: t,
//             },
//             phantom: PhantomData,
//         }
//     }

//     // TODO: Context switching working
//     pub fn with<M>(self, t: M) -> PluginRouter<TLCtx, PluginJoiner<TPlugin, MapPlugin<TLCtx, M>>>
//     where
//         M: map_plugin::AlphaMw<TLCtx>
//             + Fn(AlphaMiddlewareContext, TLCtx) -> M::Fut
//             + Send
//             + Sync
//             + 'static,
//     {
//         PluginRouter {
//             plugin: PluginJoiner {
//                 a: self.plugin,
//                 b: MapPlugin(t, PhantomData),
//             },
//             phantom: PhantomData,
//         }
//     }

//     pub async fn query<TRet: Ret, TFut: Fut<TRet>>(&self, func: impl Func<TRet, TFut>) {
//         let y = self.plugin.map(func);
//         println!("\nBUILT\n");
//         println!("{:?}\n", y.call().await);
//     }
// }

// #[cfg(test)]
// #[allow(non_snake_case)]
// mod tests {
//     use super::*;
//     use map_plugin::MapPlugin;

//     #[tokio::test]
//     async fn test_gats_middleware() {
//         // let mw = |mw| {
//         //     mw.middleware(|mw, _| async move {
//         //         let ctx = mw.ctx.clone(); // This clone is so unnessesary but Rust
//         //         Ok(mw.with_ctx((ctx, 42))) // Context switch
//         //     })
//         // };

//         // pub fn with<TNewMiddleware>(
//         //     self,
//         //     builder: impl Fn(AlphaMiddlewareBuilder<TCtx, (), ()>) -> TNewMiddleware, // TODO: Remove builder closure
//         // ) -> AlphaProcedure<
//         //     MissingResolver<TNewMiddleware::NewCtx>,
//         //     (),
//         //     AlphaMiddlewareLayerBuilder<AlphaBaseMiddleware<TCtx>, TNewMiddleware>,
//         // >
//         // where
//         //     TNewMiddleware: AlphaMiddlewareLike<LayerCtx = TCtx>,
//         // {
//         //     let mw = builder(AlphaMiddlewareBuilder(PhantomData));
//         //     AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
//         //         middleware: AlphaBaseMiddleware::new(),
//         //         mw,
//         //     })
//         // }

//         // let mw = AlphaMiddlewareBuilder<TCtx, (), ()> ::new();

//         let r = <PluginRouter>::new()
//             .plugin(MapPlugin(|| async move {
//                 println!("A");
//             }))
//             .plugin(MapPlugin(|| async move {
//                 println!("B");
//             }))
//             .query(|| async move {
//                 println!("QUERY");
//                 "Query!".to_string()
//             })
//             .await;
//     }
// }
