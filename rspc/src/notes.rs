//! This is a temporary module i'm using to store notes until v1.
//!
//! ## Rust language limitations:
//!  - Support for Serde zero-copy deserialization
//!    - We need a way to express `where F: Fn(..., I<'_>), I<'a>: Input<'a>` which to my best knowledge is impossible.
//!
//! ## More work needed:
//! - Should `Middleware::setup` return a `Result`? Probs aye?
//! - Specta more safely
//!     - [`ResolverOutput`] & [`ResolverInput`] should probs ensure the value returned and the Specta type match
//!     - That being said for `dyn Any` that could prove annoying so maybe a `Untyped<T>` escape hatch???
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
//!  - Support for Cloudflare Workers/single-threaded async runtimes. I recall this being problematic with `Send + Sync`.
//!  - Review all generics on middleware and procedure types to ensure consistent ordering.
//!     - Consistency between `TErr` and `TError`
//!  - Documentation for everything
//!  - Yank all v1 releases once 0.3.0 is out
//!
