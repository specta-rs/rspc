//! TODO: Remove this file

use std::{borrow::Cow, error, fmt, marker::PhantomData};

use futures::{stream::once, StreamExt};
use rspc::{middleware::*, procedure::*, Infallible};
use serde::Deserialize;
use specta::Type;

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

// // TODO: Make this work
// fn my_middleware_chain() {
//     // <Procedure>::builder()
//     //     .with(|ctx, _: (), next| async move {
//     //         let _result = next.exec(ctx, ()).await;
//     //     })
//     //     .with(|ctx, _: (), next| async move {
//     //         let _result = next.exec(ctx, ()).await;
//     //     })
// }

// pub fn my_middleware<N: NextExt>() -> impl Middleware<N, N::Ctx, N::Input, N::Return> {
//     mw(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         _result
//     })
// }

// pub fn my_middleware<N: NextExt>(builder: MiddlewareBuilder<N, N::Ctx, N::Input, N::Return>) {
//     builder.define(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         _result
//     })
// }

// pub fn my_middleware_with_input<N: NextExt>(
//     url: impl Into<Cow<'static, str>>,
// ) -> impl MiddlewareBuilderFn<N, N::Ctx, N::Input, N::Return> {
//     |builder| {
//         builder.define(|ctx, input, next| async move {
//             let _result = next.exec(ctx, input).await;
//             _result
//         })
//     }
// }

// pub fn my_middleware2<N: NextExt>() -> MiddlewareBuilder<N, N::Ctx, N::Input, N::Return> {
//     MiddlewareBuilder::new()
//         .state(())
//         .define(|ctx, input, next| async move {
//             let _result = next.exec(ctx, input).await;
//             _result
//         })
// }

// pub fn my_middleware3<N: NextExt>(
// ) -> MiddlewareBuilder<N, N::Ctx, N::Input, N::Return, impl Middleware> {
//     MiddlewareBuilder::new()
//         .state(())
//         .define(|ctx, input, next| async move {
//             let _result = next.exec(ctx, input).await;
//             // _result
//         })
// }

// pub fn my_middleware3<N: NextExt>() -> MiddlewareBuilder<N, (), (), i32> {
//     MiddlewareBuilder::new()
//         .state(())
//         .define(|ctx, input, next| async move {
//             let _result = next.exec(ctx, input).await;
//             // _result
//         })
// }

// TODO: N::Ctx, N::Input, N::Return
// pub fn my_middleware_with_input<N: NextExt>(
//     url: impl Into<Cow<'static, str>>,
// ) -> impl MiddlewareBuilderFn<N, N::Ctx, (), ()> {
//     |builder| {
//         // builder.define(|ctx, input, next| async move {
//         //     let _result = next.exec(ctx, input).await;
//         //     _result
//         // })
//     }
// }

// pub fn test() -> impl Middleware<TCtx> {}

// struct MiddlewareBuilder {}

// impl MiddlewareBuilder {
//     // TODO: State, etc

//     pub fn build(self) -> impl Middleware<()> {
//         todo!();
//     }
// }

// TODO: Middleware which takes args

// TODO: Use `NextExt` example
// TODO: Can `I` and `R` for the return type be infered instead? Well no cause `impl` elides them
// <TCtx, I, R>
// pub fn demo(builder: MiddlewareBuilder<Next<(), i32, i32>>) -> impl Middleware<(), (), ()> {
//     builder.define(|ctx, input, next| async move {
//         let _result = next.exec((), 42).await;
//         _result
//     })
// }

// fn logging_mw<N: NextExt>() -> Middleware<N> {
//     Middleware::new().define(|ctx, input, next| async move {
//         println!("Handling request");
//         let _result = next.exec(ctx, input).await;
//         _result
//     })
// }

// fn input_swapping<N: NextExt<Input = bool>>() -> Middleware<N, N::Ctx, u64, N::Return> {
//     Middleware::new().define(|ctx, input: u64, next| async move {
//         let _result = next.exec(ctx, input < 420).await;
//         _result
//     })
// }

// fn context_swapping<N: NextExt<Ctx = (N::, ())>>() -> Middleware<N, N::Ctx, (), N::Return> {
//     Middleware::new().define(|ctx, input: u64, next| async move {
//         let _result = next.exec(ctx, input < 420).await;
//         _result
//     })
// }

// TODO: Context swapping

// Standalone `ProcedureBuilder`'s are easier to define than middleware but can't be joined.
// fn procedure() -> ProcedureBuilder<(), Infallible, Next<(), (), ()>> {
//     <Procedure>::builder()
//     .with(|builder| {
//         builder.define(|ctx, input: (), next| async move {
//             let _result = next.exec((), true).await;
//             _result
//         })
//     })
// }

// // Curried
// fn todo0<TCtx, TErr, I, R>() -> Middleware<TErr, Next<TCtx, I, R>, Next<TCtx, I, R>> {
//     Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
// }
// fn todo_with_default<TCtx, TErr, I, R>() -> Middleware<TErr, Next<TCtx, I, R>> {
//     Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
// }

// // Reordered + default generics
// fn todo_identical_input_to_output<TCtx, TErr, I, R>() -> Middleware<TCtx, TErr, I, R> {
//     Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
// }
// fn todo_all_generics<TCtx, TErr, I, R>() -> Middleware<TCtx, TErr, I, R, I, R, TCtx> {
//     Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
// }
// fn todo_identical_ctx<TCtx, TErr, I, R>() -> Middleware<TErr, TCtx, I, R, I, R, TCtx> {
//     Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
// }

// // Flat generics
// fn todo3<TCtx, TErr, I, R>() -> Middleware<TCtx, TErr, I, R, TCtx, I, R> {
//     Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
// }
// fn todo3<TCtx, TErr, I, R>() -> Middleware<TCtx, TErr, (), R, TCtx, I, i32> {
//     Middleware::new(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         42
//     })
// }

// // Although this is *logically* incorrect I think it's more understandable
// // Basically [Incoming Context, Error Type, This layers input type (inferred back), This layers result type (inferred forward), Next layer's result]
// fn todo3<TCtx, TErr, I, R>() -> Middleware<TCtx, TErr, (), i32, R, TCtx, I> {
//     Middleware::new(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         42
//     })
// }

// TODO: This is a wacky idea
// const P: ProcedureBuilder<(), (), ()> = <Procedure>::builder().with(logging()); // P.query(...);
// const PP: Procedure = <Procedure>::builder().query(|_ctx, _input: ()| async move { 42i32 });

// [
//  Context of previous layer (`ctx`),
//  Error type,
//  The input to the middleware (`input`),
//  The result of the middleware (return type of future),
//  The context returned by the middleware (`next.exec({dis_bit}, ...)`),
//  The input to the next layer (`next.exec(..., {dis_bit})`),
//  The result of the next layer (`let _result: {dis_bit} = next.exec(...)`),
// ]
// fn todo3() -> Middleware<(), Infallible, u128, i32, (), bool, i32> {
//     Middleware::new(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         42u32
//     })
// }

// fn todo4() -> Middleware<(), Infallible, u128, i32, Next<(), bool, i32>> {
//     Middleware::new(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         42u32
//     })
// }

// fn todo5() -> Middleware<Infallible, Next<(), u128, i32>, Next<(), bool, i32>> {
//     Middleware::new(|ctx, input, next| async move {
//         let _result = next.exec(ctx, input).await;
//         42u32
//     })
// }

// fn procedure() -> ProcedureBuilder<...> {
//     Procedure::builder().with(logging())
// }
// TODO
// TODO: Can we make `procedure.query` work with some trait stuff? Probs not but worth a try.
// let todo = procedure().query(|_ctx, _input: bool| async move { 42i32 });

pub struct Node {}
pub struct Library {}

fn todo<TError, TThisCtx, TThisInput, TThisResult>(
) -> Middleware<TError, ((), TThisCtx), TThisInput, TThisResult, TThisCtx, TThisInput, TThisResult>
where
    TError: 'static,
    TThisCtx: Send + 'static,
    TThisInput: Send + 'static,
    TThisResult: Send + 'static,
{
    Middleware::new(|(_, ctx), input, next| async move { next.exec(ctx, input).await })
}
#[derive(Deserialize, Type, Debug)]
pub struct LibraryArgs<T> {
    library: String,
    args: T,
}

fn library_args<TError, TThisInput, TThisResult>() -> Middleware<
    TError,
    Node,
    LibraryArgs<TThisInput>,
    TThisResult,
    (Node, Library),
    TThisInput,
    TThisResult,
>
where
    TError: 'static,
    TThisInput: fmt::Debug + Send + 'static,
    TThisResult: fmt::Debug + Send + 'static,
{
    Middleware::new(|ctx, LibraryArgs { library, args }, next| async move {
        // TODO: Error handling if library can not be found

        next.exec((ctx, Library {}), args).await
    })
}

fn logging<TError, TThisCtx, TThisInput, TThisResult>(
) -> Middleware<TError, TThisCtx, TThisInput, TThisResult, TThisCtx, TThisInput, TThisResult>
where
    TError: 'static,
    TThisCtx: Send + 'static,
    TThisInput: fmt::Debug + Send + 'static,
    TThisResult: fmt::Debug + Send + 'static,
{
    Middleware::new(|ctx, input, next| async move {
        let input_str = format!("{input:?}");
        let start = std::time::Instant::now();
        let result = next.exec(ctx, input).await;
        println!(
            "{} {} took {:?} with input {input_str:?} and returned {result:?}",
            "QUERY",     // TODO: Make `next.meta()` work
            "todo.todo", // TODO: Make `next.meta()` work
            start.elapsed()
        );

        result
    })
}

#[tokio::main]
async fn main() {
    let procedure = Procedure::<Node>::builder()
        .with(library_args())
        .query(|(node, library), _input: u64| async move { true });
    // procedure.exec(Node {}, serde_json::Value::Null).unwrap();

    // TODO: This is why we need a 3rd `TCtx`
    // TODO: `TCtxOfFirstLayer`, `TContextOfLastLayer`, `TContextOfNextLayer`
    let procedure = Procedure::<((), Node)>::builder()
        .with(logging())
        .with(todo())
        .with(library_args())
        .query(|(node, library), _input: u64| async move { true });

    procedure
        .exec(
            ((), Node {}),
            serde_json::json!({
            "library": "test",
            "args": 42
            }),
        )
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap()
        .serialize(serde_json::value::Serializer)
        .unwrap();

    let procedure = <Procedure>::builder()
        .with(logging())
        .query(|_ctx, _input: u64| async move { true });

    // let result = procedure
    //     .exec((), serde_json::Value::Number(42u32.into()))
    //     .unwrap()
    //     .next()
    //     .await
    //     .unwrap()
    //     .unwrap()
    //     .serialize(serde_json::value::Serializer)
    //     .unwrap();
    // println!("Result: {:?}", result);

    return;

    // let procedure = <Procedure>::builder::<_, u128, _>() // TODO: Remove hardcoded `R`
    //     .with::<(), u64, bool>(logging()) // TODO: Remove hardcoded generics
    //     .query(|_ctx, _input: u64| async move { true });

    // let procedure = <Procedure>::builder().query(|_ctx, _input: ()| async move { 42i32 });

    // let procedure = <Procedure>::builder()
    //     .with(logging())
    //     .query(|_ctx, _input: ()| async move { 42i32 });

    let procedure = <Procedure>::builder()
        // .with(|ctx, input, next| async move {
        //     let _result = next.exec(ctx, input).await;
        //     _result
        // })
        .query(|_ctx, _input: ()| async move { 42i32 });

    // let router = Router::builder().procedure(procedure);

    let result = procedure
        .exec((), serde_json::Value::Null)
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap()
        .serialize(serde_json::value::Serializer)
        .unwrap();
    println!("Result: {:?}", result);

    let result = procedure
        .exec((), serde_value::Value::Unit)
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap()
        .serialize(serde_json::value::Serializer)
        .unwrap();
    println!("Result: {:?}", result);

    // let procedure =
    //     <Procedure>::builder().query(|_ctx, _input: rspc::procedure::File| async move { 42i32 });

    // let result = procedure
    //     .exec(
    //         (),
    //         rspc::procedure::File(tokio::fs::File::create("test.txt").await.unwrap()),
    //     )
    //     .unwrap()
    //     .next()
    //     .await
    //     .unwrap()
    //     .unwrap()
    //     .serialize(serde_json::value::Serializer)
    //     .unwrap();
    // println!("File Result: {:?}", result);

    let procedure = <Procedure>::builder()
        .query(|_ctx, _input: ()| async move { rspc::Stream(once(async move { 42i32 })) });

    let result = procedure
        .exec((), serde_json::Value::Null)
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|result| {
            result
                .unwrap()
                .serialize(serde_json::value::Serializer)
                .unwrap()
        })
        .collect::<Vec<_>>();

    println!("Stream Result: {:?}", result);

    // let procedure = <Procedure>::builder()
    //     .subscription(|_ctx, _input: ()| async move { once(async move { 42i32 }) });

    // let result = procedure
    //     .exec((), serde_json::Value::Null)
    //     .unwrap()
    //     .collect::<Vec<_>>()
    //     .await
    //     .into_iter()
    //     .map(|result| {
    //         result
    //             .unwrap()
    //             .serialize(serde_json::value::Serializer)
    //             .unwrap()
    //     })
    //     .collect::<Vec<_>>();

    // println!("Subscription Result: {:?}", result);

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

// pub struct LibraryArgs<T> {
//     library: String,
//     data: T,
// }
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
