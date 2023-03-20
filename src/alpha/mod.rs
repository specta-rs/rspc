#![allow(unused)] // TODO: Remove once this stuff has been stabilized

mod middleware;
mod procedure;
mod procedure_like;
mod router;
mod rspc;
mod error;

pub use self::rspc::*;
pub use middleware::*;
pub use procedure::*;
pub use procedure_like::*;
pub use router::*;
pub use error::*;

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, path::PathBuf, time::Duration};

    use async_stream::stream;
    use serde::{de::DeserializeOwned, Serialize};
    use specta::Type;
    use tokio::time::sleep;

    use crate::{
        alpha::{
            procedure::AlphaProcedure, AlphaBaseMiddleware, Marker, MiddlewareArgMapper, Mw,
            ProcedureLike, ResolverFunction,
        },
        internal::ProcedureKind,
        RequestLayer,
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
            .procedure(
                "todo",
                t.with(|mw| {
                    mw.middleware(|mw, _| async move {
                        let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
                        Ok(mw.with_state(state))
                    })
                    .resp(|state, result| async move {
                        println!(
                            "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
                            state.0, state.1, state.2, result
                        );
                        Ok(result)
                    })
                })
                .query(|ctx, _: ()| {
                    println!("TODO: {:?}", ctx);
                    Ok(())
                }),
                // .meta(()),
            )
            .procedure(
                "todo2",
                t.with(|mw| {
                    mw.middleware(|mw, _| async move {
                        let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
                        Ok(mw.with_state(state))
                    })
                    .resp(|state, result| async move {
                        println!(
                            "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
                            state.0, state.1, state.2, result
                        );
                        Ok(result)
                    })
                })
                .with(|mw| {
                    mw.middleware(|mw, _| async move {
                        let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
                        Ok(mw.with_state(state))
                    })
                    .resp(|state, result| async move {
                        println!(
                            "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
                            state.0, state.1, state.2, result
                        );
                        Ok(result)
                    })
                })
                .query(|ctx, _: ()| {
                    println!("TODO: {:?}", ctx);
                    Ok(())
                }),
                // .meta(()),
            )
            .procedure(
                "todo3",
                t.query(|ctx, _: ()| {
                    println!("TODO: {:?}", ctx);
                    Ok(())
                }),
            )
            .procedure(
                "demoSubscriptions",
                t.subscription(|_ctx, _args: ()| {
                    stream! {
                        println!("Client subscribed to 'pings'");
                        for i in 0..5 {
                            println!("Sending ping {}", i);
                            yield "ping".to_string();
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                })
            )
            // TODO: This shouldn't work
            .procedure(
                "veryInvalid",
                t.query(|_ctx, _args: ()| {
                    stream! {
                        println!("Client subscribed to 'pings'");
                        for i in 0..5 {
                            println!("Sending ping {}", i);
                            yield "ping".to_string();
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }),
            )
            .compat();

        r.export_ts(PathBuf::from("./demo.bindings.ts")).unwrap();
    }

    #[test]
    fn test_context_switching() {
        const t: Rspc = Rspc::new();

        let p = t
            .with(|mw| {
                mw.middleware(|mw, _| async move {
                    let ctx = mw.ctx.clone(); // This clone is so unnessesary but Rust
                    Ok(mw.with_ctx((ctx, 42))) // Context switch
                })
            })
            .query(|ctx, _: ()| {
                let (ctx, num) = ctx; // Typecheck the context switch
                Ok(())
            });

        fn demo() -> impl ProcedureLike<((), i32)> {
            t.with(|mw| {
                mw.middleware(|mw, _| async move {
                    let ctx = mw.ctx.clone(); // This clone is so unnessesary but Rust
                    Ok(mw.with_ctx((ctx, 42))) // Context switch
                })
            })
        }

        let p = demo().query(|ctx, _: ()| {
            println!("TODO: {:?}", ctx);
            let _ = ctx.0; // Test Rust inference is working
            Ok(())
        });
    }

    #[test]
    fn with_middleware_from_func() {
        pub fn library<TLayerCtx, TPrevMwMapper>() -> impl Mw<TLayerCtx, TPrevMwMapper, NewLayerCtx = (TLayerCtx, i32)>
        where
            TLayerCtx: Send + Sync + Clone + 'static,
            TPrevMwMapper: MiddlewareArgMapper + Send + Sync + 'static,
        {
            |mw| mw.middleware(|mw, _| async move { Ok(mw.map_ctx(|ctx| (ctx, 42))) })
        }

        let p = t.with(library()).query(|ctx, _: ()| {
            println!("TODO: {:?}", ctx);
            let _ = ctx.0; // Test Rust inference is working
            Ok(())
        });

        let p = t.with(library()).with(library()).query(|ctx, _: ()| {
            println!("TODO: {:?}", ctx);
            let ((a, b), c) = ctx; // Test Rust inference is working
            Ok(())
        });
    }

    #[test]
    fn middleware_args() {
        pub struct LibraryArgsMap;

        impl MiddlewareArgMapper for LibraryArgsMap {
            type Input<T> = (T, i32) 
            where
                T: DeserializeOwned + Type + 'static;
            type Output<T> = T where T: Serialize;
            type State = i32;

            fn map<T: Serialize + DeserializeOwned + Type + 'static>(
                arg: Self::Input<T>,
            ) -> (Self::Output<T>, Self::State) {
                (arg.0, arg.1)
            }
        }

        let p = t
            .with(|mw| {
                mw.args::<LibraryArgsMap>()
                .middleware(|mw, arg| async move {
                    println!("{:?}", ()); // TODO: Access args

                    Ok(mw.map_ctx(|ctx| (ctx, 42)))
                })
            })
            .query(|ctx, _: ()| {
                println!("TODO: {:?}", ctx);
                let _ = ctx.0; // Test Rust inference is working
                Ok(())
            });
    }

    #[test]
    fn multiple_middleware_args() {
        pub struct DoubleTupleMapper;

        impl MiddlewareArgMapper for DoubleTupleMapper {
            type Input<T> = (T, Self::State) 
            where
                T: DeserializeOwned + Type + 'static;
            type Output<T> = T where T: Serialize;
            type State = ((), ());

            fn map<T: Serialize + DeserializeOwned + Type + 'static>(
                arg: Self::Input<T>,
            ) -> (Self::Output<T>, Self::State) {
                (arg.0, ((), ()))
            }
        }

        pub struct TripleTupleMapper;

        impl MiddlewareArgMapper for TripleTupleMapper {
            type Input<T> = (T, Self::State) 
            where
                T: DeserializeOwned + Type + 'static;
            type Output<T> = T where T: Serialize;
            type State = ((), (), ());

            fn map<T: Serialize + DeserializeOwned + Type + 'static>(
                arg: Self::Input<T>,
            ) -> (Self::Output<T>, Self::State) {
                (arg.0, ((), (), ()))
            }
        }

        let p = t
            .with(|mw| {
                mw.args::<DoubleTupleMapper>()
                .middleware(|mw, (_one, _two)| async move {
                    println!("{:?}", ()); // TODO: Access args

                    Ok(mw.map_ctx(|ctx| (ctx, 42)))
                })
            })
            .with(|mw| {
                mw.args::<TripleTupleMapper>()
                .middleware(|mw, (_one, _two, _three)| async move {
                    println!("{:?}", ()); // TODO: Access args

                    Ok(mw.map_ctx(|ctx| ctx))
                })
            })
            .query(|ctx, a: i32| {
                println!("TODO: {:?}", ctx);
                let _ = ctx.0; // Test Rust inference is working
                Ok(())
            });

            let r = t.router().procedure("demo", p).compat().export_ts(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./demo2.bindings.ts")).unwrap();
    }
}
