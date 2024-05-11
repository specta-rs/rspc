//! This is a temporary module i'm using to store notes until v1.
//!
//! ## Rust language limitations:
//!  - Support for Serde zero-copy deserialization
//!    - We need a way to express `where F: Fn(..., I<'_>), I<'a>: Input<'a>` which to my best knowledge is impossible.
//!
//! ## More work needed:
//!  - Specta more safely
//!     - [`ResolverOutput`] & [`ResolverInput`] should probs ensure the value returned and the Specta type match
//!     - That being said for `dyn Any` that could prove annoying so maybe a `Untyped<T>` escape hatch???
//!  - new middleware system
//!     - downcast/upcast the input and context between procedures
//!     - export the input type of the first middleware (not the procedure like it would be now)
//!     - `Procedure` needs to return `TCtx` not `TNewCtx` -> Right now they are tied together on [`ProcedureBuilder`]
//!  - handling of result types is less efficient that it could be
//!     - If it can only return one type can be do the downcast/deserialization magic internally.
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
//!  - Yank all v1 releases once 0.3.0 is out
//!
