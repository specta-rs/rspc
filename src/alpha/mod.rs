#![allow(unused)] // TODO: Remove once this stuff has been stabilized

mod middleware;
mod procedure;
mod procedure_like;
mod router;
mod rspc;

pub use self::rspc::*;
pub use middleware::*;
pub use procedure::*;
pub use procedure_like::*;
pub use router::*;

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, path::PathBuf};

    use serde::de::DeserializeOwned;
    use specta::Type;

    use crate::{
        alpha::{
            procedure::AlphaProcedure, AlphaBaseMiddleware, Marker, Mw, ProcedureLike,
            ResolverFunction,
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
                    mw.middleware(|mw| async move {
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
                    mw.middleware(|mw| async move {
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
                    mw.middleware(|mw| async move {
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
            .compat();

        r.export_ts(PathBuf::from("./demo.bindings.ts")).unwrap();
    }

    // fn admin_middleware() -> impl Middleware {} // TODO: basic middleware + context switching
    // TODO: Allowing a router to take parameters -> Will require proxy syntax on frontend
    // TODO: Internally storing those as `fn()` instead of `impl Fn()` (Basically using a `Cow` for functions)??

    // TODO: Maybe making a macro to do this without so many internal APIs, lmao!
    fn test_crazy_spacedrive_stuff() {
        pub type Context = ();
        pub type Db = i32;
        pub struct Library {
            db: i32,
        }
        pub struct SdRspc {
            db: i32,
        };

        // TODO: THIS SYSTEM DOESN'T HANDLE `t.use(...).query(...)` it only does `t.query(...)`

        pub struct LibraryResolver<F>(F, Db);

        pub struct BetterMarker<A, B, C>(PhantomData<(A, B, C)>);
        impl<
                TLayerCtx,
                TArg,
                TResult,
                TResultMarker,
                F: Fn(TLayerCtx, TArg, Db) -> TResult + Send + Sync + 'static,
            > ResolverFunction<TLayerCtx, BetterMarker<TArg, TResult, TResultMarker>>
            for LibraryResolver<F>
        where
            TArg: DeserializeOwned + Type,
            TResult: RequestLayer<TResultMarker>,
        {
            type Arg = TArg;
            type Result = TResult;
            type ResultMarker = TResultMarker;

            fn exec(&self, ctx: TLayerCtx, arg: Self::Arg) -> Self::Result {
                self.0(ctx, arg, self.1)
            }
        }

        impl SdRspc {
            pub fn query<R, TArg, TResult, TResultMarker>(
                &self,
                builder: R,
            ) -> AlphaProcedure<
                Context,
                Context,
                LibraryResolver<R>,
                BetterMarker<TArg, TResult, TResultMarker>,
                (),
                AlphaBaseMiddleware<Context>,
            >
            where
                R: Fn(Context, TArg, Db) -> TResult + Send + Sync + 'static,
                TArg: DeserializeOwned + Type,
                TResult: RequestLayer<TResultMarker>,
            {
                AlphaProcedure::new_from_resolver(
                    ProcedureKind::Query,
                    LibraryResolver(builder, 42),
                )
            }
        }

        const t: SdRspc = SdRspc { db: 42 };

        let p = t.query(|ctx, _: (), db| {
            println!("TODO: {:?}", ctx);
            Ok(())
        });

        let p = t.query(|ctx, _: (), db| {
            println!("TODO: {:?}", ctx);
            Ok(())
        });
    }

    fn test_context_switching() {
        const t: Rspc = Rspc::new();

        let p = t
            .with(|mw| {
                mw.middleware(|mw| async move {
                    let ctx = mw.ctx.clone(); // This clone is so unnessesary but Rust
                    Ok(mw.with_ctx((ctx, 42))) // Context switch
                })
            })
            .query(|ctx, _: ()| {
                let (ctx, num) = ctx; // Typecheck the context switch
                Ok(())
            });

        fn demo() -> impl ProcedureLike<(), ((), i32)> {
            t.with(|mw| {
                mw.middleware(|mw| async move {
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

    fn with_middleware_from_func() {
        pub fn library<TLayerCtx>() -> impl Mw<TLayerCtx, NewLayerCtx = (TLayerCtx, i32)>
        where
            TLayerCtx: Send + Sync + Clone + 'static,
        {
            |mw| mw.middleware(|mw| async move { Ok(mw.map_ctx(|ctx| (ctx, 42))) })
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

    // fn middleware_args() {
    //     let p = t
    //         .with(|mw| {
    //             mw.args().middleware(|mw| async move {
    //                 println!("{:?}", ()); // TODO: Access args

    //                 Ok(mw.map_ctx(|ctx| (ctx, 42)).map_arg())
    //             })
    //         })
    //         .query(|ctx, _: ()| {
    //             println!("TODO: {:?}", ctx);
    //             let _ = ctx.0; // Test Rust inference is working
    //             Ok(())
    //         });
    // }

    // TODO: `LibraryArgs<T>` style system with middleware
}
