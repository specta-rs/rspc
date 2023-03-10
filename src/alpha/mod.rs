#![allow(unused)] // TODO: Remove once this stuff has been stabilized

mod procedure;
mod router;
mod rspc;

pub use self::rspc::*;
pub use procedure::*;
pub use router::*;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Rspc;

    #[allow(non_upper_case_globals)]
    const t: Rspc<()> = Rspc::new();

    #[test]
    fn test_alpha_api() {
        // TODO: Get Context switching?
        // TODO: `TMeta` working on a procedure level?
        // TODO: Remove `TMeta` from old API?

        let r = t
            .router()
            .procedure(
                "todo",
                t.query(|tt| {
                    tt(|ctx, _: ()| {
                        println!("TODO: {:?}", ctx);
                        Ok(())
                    })
                })
                .meta(()),
            )
            .procedure(
                "todo2",
                t.query(|tt| {
                    tt(|ctx, _: ()| {
                        println!("TODO: {:?}", ctx);
                        Ok(())
                    })
                }),
            )
            .compat();

        r.export_ts(PathBuf::from("./demo.bindings.ts")).unwrap();
    }

    // TODO: `.with()` syntax for middleware that lets you stack them
    // const t: Rspc<()> = Rspc::new_with_mw(); // TODO: making something like this work?
    // fn admin_middleware() -> impl Middleware {} // TODO: basic middleware + context switching
    // TODO: Allowing a router to take parameters -> Will require proxy syntax on frontend
    // TODO: Showing a const router? -> Can we can be type erased at that point -> Internally storing those as `fn()` instead of `impl Fn()` (Basically using a `Cow` for functions)??
}
