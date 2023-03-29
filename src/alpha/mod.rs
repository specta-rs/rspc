// TODO: Refactor type names though this whole package cause it's currently pretty messy
#![allow(deprecated)] // TODO: Remove once stuff is stabilized
#![allow(unused)] // TODO: Remove once this stuff has been stabilized

mod error;
mod layer;
mod middleware;
mod middleware_old;
pub mod mw2; // TODO: `pub use mw2::*;` on this
mod procedure;
mod procedure_like;
mod resolver_result;
mod router;
mod router_builder_like;
mod rspc;

pub use self::rspc::*;
pub use error::*;
pub use layer::*;
pub use middleware::*;
pub use middleware_old::*;
pub use procedure::*;
pub use procedure_like::*;
pub use resolver_result::*;
pub use router::*;
pub use router_builder_like::*;

pub use crate::alpha_stable::*;

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, path::PathBuf, time::Duration};

    use async_stream::stream;
    use serde::{de::DeserializeOwned, Serialize};
    use specta::Type;
    use tokio::time::sleep;

    use crate::{
        alpha::{
            procedure::AlphaProcedure, AlphaBaseMiddleware, MiddlewareArgMapper, Mw, ProcedureLike,
        },
        internal::ProcedureKind,
    };

    use super::Rspc;

    #[allow(non_upper_case_globals)]
    const t: Rspc<()> = Rspc::new();

    #[test]
    fn test_alpha_api() {
        // TODO: Get Context switching?
        // TODO: `TMeta` working on a procedure level?

        let r = t
            .router()
            // .procedure(
            //     "todo",
            //     t.with(|mw, ctx| async move { mw.next(ctx) })
            //         .query(|ctx, _: ()| {
            //             println!("TODO: {:?}", ctx);
            //             Ok(())
            //         }),
            // )
            // .procedure(
            //     "todo2",
            //     t.with(|mw, ctx| async move {
            //         let msg = format!(
            //             "[LOG] req='{:?}' ctx='{:?}' input='{:?}'",
            //             mw.req, ctx, mw.input
            //         );
            //         mw.next(ctx).resp(|result| async move {
            //             println!("{msg} result='{result:?}'");
            //             result
            //         })
            //     })
            //     // TODO: Make this work
            //     // .with(|mw, ctx| async move {
            //     //     let msg = format!(
            //     //         "[LOG2] req='{:?}' ctx='{:?}' input='{:?}'",
            //     //         mw.req, ctx, mw.input
            //     //     );
            //     //     mw.next(ctx).resp(|result| async move {
            //     //         println!("{msg} result='{result:?}'");
            //     //         result
            //     //     })
            //     // })
            //     .query(|ctx, _: ()| {
            //         println!("TODO: {:?}", ctx);
            //         Ok(())
            //     }),
            // )
            // .procedure(
            //     "todo3",
            //     t.query(|ctx, _: ()| {
            //         println!("TODO: {:?}", ctx);
            //         Ok(())
            //     }),
            // )
            // .procedure(
            //     "demoSubscriptions",
            //     t.subscription(|_ctx, _args: ()| {
            //         stream! {
            //             println!("Client subscribed to 'pings'");
            //             for i in 0..5 {
            //                 println!("Sending ping {}", i);
            //                 yield "ping".to_string();
            //                 sleep(Duration::from_secs(1)).await;
            //             }
            //         }
            //     }),
            // )
            .compat();

        r.export_ts(PathBuf::from("./demo.bindings.ts")).unwrap();
    }

    // TODO: Fix all of these tests

    // #[test]
    // fn test_context_switching() {
    //     const t: Rspc = Rspc::new();

    //     let p = t
    //         .with(|mw| {
    //             mw.middleware(|mw, _| async move {
    //                 let ctx = mw.ctx.clone(); // This clone is so unnessesary but Rust
    //                 Ok(mw.with_ctx((ctx, 42))) // Context switch
    //             })
    //         })
    //         .query(|ctx, _: ()| {
    //             let (ctx, num) = ctx; // Typecheck the context switch
    //             Ok(())
    //         });

    //     fn demo() -> impl ProcedureLike<LayerCtx = ((), i32)> {
    //         t.with(|mw| {
    //             mw.middleware(|mw, _| async move {
    //                 let ctx = mw.ctx.clone(); // This clone is so unnessesary but Rust
    //                 Ok(mw.with_ctx((ctx, 42))) // Context switch
    //             })
    //         })
    //     }

    //     let p = demo().query(|ctx, _: ()| {
    //         println!("TODO: {:?}", ctx);
    //         let _ = ctx.0; // Test Rust inference is working
    //         Ok(())
    //     });
    // }

    // #[test]
    // fn with_middleware_from_func() {
    //     pub fn library<TLayerCtx, TPrevMwMapper>() -> impl Mw<TLayerCtx, TPrevMwMapper, NewLayerCtx = (TLayerCtx, i32)>
    //     where
    //         TLayerCtx: Send + Sync + Clone + 'static,
    //         TPrevMwMapper: MiddlewareArgMapper,
    //     {
    //         |mw| mw.middleware(|mw, _| async move { Ok(mw.map_ctx(|ctx| (ctx, 42))) })
    //     }

    //     let p = t.with(library()).query(|ctx, _: ()| {
    //         println!("TODO: {:?}", ctx);
    //         let _ = ctx.0; // Test Rust inference is working
    //         Ok(())
    //     });

    //     let p = t.with(library()).with(library()).query(|ctx, _: ()| {
    //         println!("TODO: {:?}", ctx);
    //         let ((a, b), c) = ctx; // Test Rust inference is working
    //         Ok(())
    //     });
    // }

    // #[test]
    // fn middleware_args() {
    //     pub struct LibraryArgsMap;

    //     impl MiddlewareArgMapper for LibraryArgsMap {
    //         type Input<T> = (T, i32)
    //         where
    //             T: DeserializeOwned + Type + 'static;
    //         type Output<T> = T where T: Serialize;
    //         type State = i32;

    //         fn map<T: Serialize + DeserializeOwned + Type + 'static>(
    //             arg: Self::Input<T>,
    //         ) -> (Self::Output<T>, Self::State) {
    //             (arg.0, arg.1)
    //         }
    //     }

    //     let p = t
    //         .with(|mw| {
    //             mw.args::<LibraryArgsMap>()
    //             .middleware(|mw, arg| async move {
    //                 println!("{:?}", ()); // TODO: Access args

    //                 Ok(mw.map_ctx(|ctx| (ctx, 42)))
    //             })
    //         })
    //         .query(|ctx, _: ()| {
    //             println!("TODO: {:?}", ctx);
    //             let _ = ctx.0; // Test Rust inference is working
    //             Ok(())
    //         });
    // }

    // #[test]
    // fn multiple_middleware_args() {
    //     pub struct DoubleTupleMapper;

    //     impl MiddlewareArgMapper for DoubleTupleMapper {
    //         type Input<T> = (T, Self::State)
    //         where
    //             T: DeserializeOwned + Type + 'static;
    //         type Output<T> = T where T: Serialize;
    //         type State = ((), ());

    //         fn map<T: Serialize + DeserializeOwned + Type + 'static>(
    //             arg: Self::Input<T>,
    //         ) -> (Self::Output<T>, Self::State) {
    //             (arg.0, ((), ()))
    //         }
    //     }

    //     pub struct TripleTupleMapper;

    //     impl MiddlewareArgMapper for TripleTupleMapper {
    //         type Input<T> = (T, Self::State)
    //         where
    //             T: DeserializeOwned + Type + 'static;
    //         type Output<T> = T where T: Serialize;
    //         type State = ((), (), ());

    //         fn map<T: Serialize + DeserializeOwned + Type + 'static>(
    //             arg: Self::Input<T>,
    //         ) -> (Self::Output<T>, Self::State) {
    //             (arg.0, ((), (), ()))
    //         }
    //     }

    //     let p = t
    //         .with(|mw| {
    //             mw.args::<DoubleTupleMapper>()
    //             .middleware(|mw, (_one, _two)| async move {
    //                 println!("{:?}", ()); // TODO: Access args

    //                 Ok(mw.map_ctx(|ctx| (ctx, 42)))
    //             })
    //         })
    //         .with(|mw| {
    //             mw.args::<TripleTupleMapper>()
    //             .middleware(|mw, (_one, _two, _three)| async move {
    //                 println!("{:?}", ()); // TODO: Access args

    //                 Ok(mw.map_ctx(|ctx| ctx))
    //             })
    //         })
    //         .query(|ctx, a: i32| {
    //             println!("TODO: {:?}", ctx);
    //             let _ = ctx.0; // Test Rust inference is working
    //             Ok(())
    //         });

    //         let r = t.router().procedure("demo", p).compat().export_ts(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./demo2.bindings.ts")).unwrap();
    // }
}
