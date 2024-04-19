//! TODO: Remove this file

use rspc::procedure::*;

// TODO: async

// TODO: Typesafe error handling (Using `.error`)
// TODO: Return types (either Serde or Stream or files)
// TODO: Convert this file into unit tests

fn main() {
    // Everything here:
    // - Runs top to bottom (using `next.exec` to continue to chain)
    // - The resolver *must* be defined last
    // These semantics match the current rspc middleware system from v1 alphas.

    // Just a procedure
    <Procedure>::builder().query(|_ctx, _: ()| 42i32);
    Procedure::<i32>::builder().query(|_ctx, _: ()| 42i32);
    Procedure::builder().query(|ctx: (), _: ()| 42i32);

    // Single middleware
    <Procedure>::builder()
        .with(|ctx, _: (), next| {
            let _result = next.exec(ctx, ());
        })
        .query(|_ctx, _: ()| 42i32);

    // Confirm result type behavior if we have multiple middleware
    <Procedure>::builder() // (bool, (&str, i32))
        .with(|ctx, _: (), next| {
            let result = next.exec(ctx, ()); // (&str, i32)
            (true, result)
        })
        .with(|ctx, _: (), next| {
            let result = next.exec(ctx, ()); // i32
            ("", result)
        })
        .query(|_ctx, _: ()| 42i32);

    // Confirm input type behavior if we have multiple middleware
    <Procedure>::builder()
        .with(library_args())
        .with(|ctx, input: (bool, (i32, ())), next| next.exec(ctx, input.1))
        .with(|ctx, input, next| next.exec(ctx, input.1))
        .query(|_ctx, _| 42i32);

    // Confirm context type behavior
    <Procedure>::builder()
        .with(|ctx, input, next| next.exec((true, ctx), input))
        .with(|ctx, input, next| next.exec(("", ctx), input))
        .query(|_ctx, _: ()| 42i32);

    // What if we don't call `next`
    // - This can be a problem with unconstraining the generic so it needs docs for developer but it's not a make or break thing.
    <Procedure>::builder()
        // TODO: `Next`'s `TCtx` default is kinda pointless here cause it's just `()` not the users one
        .with(|_, _: (), next: Next<_, _>| "No cringe past this point")
        .query(|_, _: ()| 42i32);

    // TODO: sort this out
    // <Procedure>::builder()
    //     .with(|_, _, next| "No cringe past this point")
    //     .query(|_, _| 42i32);
}

pub struct LibraryArgs<T> {
    library: String,
    data: T,
}
// TODO: middleware helpers to make this easier
fn library_args<TCtx, T, NextR>() -> impl Fn(TCtx, LibraryArgs<T>, Next<NextR, T, TCtx>) -> NextR {
    |ctx, input, next| next.exec(ctx, input.data)
}
