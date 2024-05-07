//! TODO: Remove this file

use std::any::Any;

use erased_serde::Serialize;
use rspc::procedure::*;
use serde_json::Value;

// TODO: Fix library args example

// TODO: TCtx vs TNewCtx
// TODO: Typesafe error handling (Using `.error`)
// TODO: Return types (either Serde or Stream or files)

// TODO: Runtime part working

// TODO: Syntax testing with `trybuild` & convert this file into unit tests

// fn library_args<TCtx, T, NextR>(
// ) -> impl Fn(TCtx, LibraryArgs<T>, Next<NextR, T, TCtx>) -> Future<Output = NextR> {
//     |ctx, input, next| async move { next.exec(ctx, input.data).await }
// }

// pub fn error_only<N: NextGenerics>() -> impl Middleware<N> {
//     // You must always constrain the input types (but they can be generics)
//     mw(|ctx: (), input: (), next| async move {
//         // We don't touch `next` so Rust doesn't care that we don't know it's generics
//     })
// }

// pub fn any<TCtx, TInput, N: NextGenerics<Ctx = TCtx, Input = TInput>>() -> impl Middleware<N> {
//     mw(|ctx: TCtx, input: TInput, next| async move {
//         // As we touch `next` we need to constrain it. `NextTrait<Ctx = ..., TInput = ...`
//         let _result = next.exec(ctx, input).await;
//     })
// }

// pub fn todo_register<N: NextGenerics<Ctx = (), Input = ()>>() -> impl Middleware<N> {
//     mw(|ctx: (), input: (), next| async move {
//         let _result = next.exec(ctx, input).await;
//     })
// }

// pub fn doesnt_call_next<TCtx, TInput, TNextResult>() -> impl Middleware {
//     mw(|ctx: TCtx, input: TInput, next: PlaceholderNext| async move {})
// }

/// A middleware that takes in anything and returns it
// pub fn any<TCtx, TInput, TNextResult>() -> impl Middleware<Next<i32, TInput, TCtx>> {
//     mw(
//         // TODO: Generic return type
//         |ctx: TCtx, input: TInput, next: Next<i32, TInput, TCtx>| async move {
//             let _result = next.exec(ctx, input).await;
//         },
//     )
// }

// pub fn hardcoded<T>() -> impl Middleware {
//     mw(|ctx: (), input: T, next| async move {
//         let _result: i32 = next.exec(ctx, input).await;
//     })
// }

// pub fn todo_plz_work() -> impl Middleware<Next<i32, (), ()>> {
//     mw(
//         // TODO: Generic return type
//         |ctx: (), input: (), next: Next<i32, (), ()>| async move {
//             let _result = next.exec(ctx, input).await;
//         },
//     )
// }

// .register(|ctx| {
//     println!("Run during router builder!");
//     // ctx.procedure_name;
//     // ctx.state.insert(todo!());
//     |ctx, _: (), next| async move {
//         let _result = next.exec(ctx, ()).await;
//     }
// })
// .with(mw(|ctx, _: (), next| async move {
//     let _result = next.exec(ctx, ()).await;
// }))
// .with(error_only())
// .with(todo_plz_work())

pub trait Format {
    // TODO: This can't take `T` cause it would make it not-dyn safe
    fn serialize(&mut self, value: &dyn Any) {
        // TODO: dyn Any -> Serialize????
        // *self =
    }
}

pub struct SerdeJson(serde_json::Value);

impl Format for SerdeJson {}

#[tokio::main]
async fn main() {
    // TODO: The format is local to the procedure which is kinda problematic as you can only have one per-router (Eg. no Json and FormData)

    let procedure = <Procedure>::builder().query(|_ctx, _input: ()| async move { 42i32 });

    match procedure.exec((), ()) {
        ProcedureExecResult::Future(fut) => {
            let result = fut
                .await
                .unwrap()
                .serialize(serde_json::value::Serializer)
                .unwrap();
            println!("Result: {:?}", result);
        }
        ProcedureExecResult::Stream(_) => unreachable!(),
    }

    // TODO: BREAK

    // <Procedure>::builder()
    //     // .with(
    //     //     MiddlewareBuilder::builder().with(|ctx, _: (), next| async move {
    //     //         let _result = next.exec(ctx, ()).await;
    //     //     }),
    //     // )
    //     .query(|_ctx, _: ()| async move { 42i32 });

    // Everything here:
    // - Runs top to bottom (using `next.exec` to continue to chain)
    // - The resolver *must* be defined last
    // These semantics match the current rspc middleware system from v1 alphas.

    // Just a procedure
    // <Procedure>::builder().query(|_ctx, _: ()| async move { 42i32 });
    // Procedure::<i32>::builder().query(|_ctx, _: ()| async move { 42i32 });
    // Procedure::builder().query(|ctx: (), _: ()| async move { 42i32 });

    // Single middleware
    // <Procedure>::builder()
    //     .with(mw(|ctx, _: (), next| async move {
    //         let _result = next.exec(ctx, ()).await;
    //     }))
    //     .query(|_ctx, _: ()| async move { 42i32 });

    // <Procedure>::builder()
    //     // .with(
    //     //     MiddlewareBuilder::builder()
    //     //         .state(())
    //     //         .start(|| println!("Setting up!"))
    //     //         .with(|ctx, _: (), next| async move {
    //     //             let _result = next.exec(ctx, ()).await;
    //     //         }),
    //     // )
    //     .query(|_ctx, _: ()| async move { 42i32 });

    // // Confirm result type behavior if we have multiple middleware
    // <Procedure>::builder() // (bool, (&str, i32))
    //     .with(|ctx, _: (), next| async move {
    //         let result = next.exec(ctx, ()).await; // (&str, i32)
    //         (true, result)
    //     })
    //     .with(|ctx, _: (), next| async move {
    //         let result = next.exec(ctx, ()); // i32
    //         ("", result)
    //     })
    //     .query(|_ctx, _: ()| async move { 42i32 });

    // // Confirm input type behavior if we have multiple middleware
    // // <Procedure>::builder()
    // //     .with(library_args())
    // //     .with(|ctx, input: (bool, (i32, ())), next| next.exec(ctx, input.1))
    // //     .with(|ctx, input, next| next.exec(ctx, input.1))
    // //     .query(|_ctx, _| async move { 42i32 });

    // // Confirm context type behavior
    // <Procedure>::builder()
    //     .with(|ctx, input, next| next.exec((true, ctx), input))
    //     .with(|ctx, input, next| next.exec(("", ctx), input))
    //     .query(|_ctx, _: ()| async move { 42i32 });

    // // What if we don't call `next`
    // // - This can be a problem with unconstraining the generic so it needs docs for developer but it's not a make or break thing.
    // <Procedure>::builder()
    //     .with(|_, _: (), next| async move { "No cringe past this point" })
    //     .query(|_: (), _: ()| async move { 42i32 });

    // <Procedure>::builder()
    //     .with(|_, _: (), next| {
    //         if true {
    //             "skip"
    //         } else {
    //             let _result = next.exec(93, ());
    //             "ok"
    //         }
    //     })
    //     .query(|_, _: ()| 42i32);
}

pub struct LibraryArgs<T> {
    library: String,
    data: T,
}
// TODO: middleware helpers to make this easier
// fn library_args<TCtx, T, NextR>(
// ) -> impl Fn(TCtx, LibraryArgs<T>, Next<NextR, T, TCtx>) -> Future<Output = NextR> {
//     |ctx, input, next| async move { next.exec(ctx, input.data).await }
// }

// fn library_args<TCtx, NextI, NextR>() -> impl Middleware<TCtx, LibraryArgs<NextI>, NextI, NextR> {
//     // TODO: Avoid the hardcoded type on `next`
//     mw(|ctx, input, next: Next<NextR, NextI, TCtx>| async move { next.exec(ctx, input.data).await })
// }

// fn mw<TCtx, I, NextI, NextR>(
//     mw: impl Middleware<TCtx, I, NextI, NextR>,
// ) -> impl Middleware<TCtx, I, NextI, NextR> {
//     mw
// }

// pub trait Middleware<TCtx, I, NextI, NextR> {
//     // type Result;
//     // TODO: Associated types for all the stuff

//     fn call(&self) -> impl Future;
// }

// impl<TCtx, I, NextI, NextR, F, Fu> Middleware<TCtx, I, NextI, NextR> for F
// where
//     F: Fn(TCtx, I, Next<NextR, NextI, TCtx>) -> Fu,
//     Fu: Future,
// {
//     fn call(&self) -> impl Future {
//         async move {
//             todo!();
//         }
//     }
// }
