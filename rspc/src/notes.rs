//! This is a temporary module i'm using to store notes until v1.
//!
//! ## Rust language limitations:
//!  - Support for Serde zero-copy deserialization
//!    - We need a way to express `where F: Fn(..., I<'_>), I<'a>: Input<'a>` which to my best knowledge is impossible.
//!
//! ## More work needed:
//!  - Make `Middleware::setup` work.
//!  - The order of `.error` in the procedure chain and it's affects on middleware
//!     - Can we allow changing it between middleware if an `Into` impl exists?
//!  - Error handling with middleware
//!  - Boxed or non-boxed procedure. Maybe using a different constructor and default generics?
//!  - Can `R` be replaced with something like middleware chains???
//!     - For something like logging it would be nice to apply it to the router (I don't like this) or have a base procedure concept like tRPC.
//!     - We could approach this from `Middleware::with` & `Middleware::error` or `ProcedureBuilder::merge`. I suspect the latter is the better approach tbh.
//! - Should `Middleware::setup` return a `Result`? Probs aye?
//! - Specta more safely
//!     - [`ResolverOutput`] & [`ResolverInput`] should probs ensure the value returned and the Specta type match
//!     - That being said for `dyn Any` that could prove annoying so maybe a `Untyped<T>` escape hatch???
//!  - new middleware system
//!     - downcast/upcast the input and context between procedures
//!     - export the input type of the first middleware (not the procedure like it would be now)
//!     - `Procedure` needs to return `TCtx` not `TNewCtx` -> Right now they are tied together on [`ProcedureBuilder`]
//!  - handling of result types is less efficient that it could be
//!     - If it can only return one type can be do 'the downcast/deserialization magic internally.
//!  - new Rust diagnostic stuff on all the traits
//!  - Handle panics within procedures
//!  - `ResolverOutput` trait oddities
//!     - `rspc::Stream` within a `rspc::Stream` will panic at runtime
//!     - Am I happy with `Output::into_procedure_stream`? It's low key cringe but it might be fine.
//!  - `ProcedureInput` vs `ResolverInput` typesafety
//!    - You can implement forget to implement `ProcedureInput` for an `ResolverInput` type.
//!    - For non-serde types really `ResolverInput` and `ProcedureInput` are the same, can we express that? Probs not without specialization and markers.
//!  - Are the restricted return types fine
//!     - We have heavily restricted return types to reduce marker traits and help the compiler with errors.
//!     - Was this a good move?
//!  - Two-way communication primitive. Maybe a [`ProcedureBuilder::socket`]??
//!  - Can we drop second generic for middleware and constrain the associated type instead???
//!     - I don't know if this is a good idea or not but worth considering.
//!  - Can we abstract a middleware chain. All `register`, `with` and `error` methods abstracted out into a dedicated function.
//!  - Support for Cloudflare Workers/single-threaded async runtimes. I recall this being problematic with `Send + Sync`.
//!  - Review all generics on middleware and procedure types to ensure consistent ordering.
//!     - Consistency between `TErr` and `TError`
//!  - Documentation for everything
//!  - Yank all v1 releases once 0.3.0 is out
//!
