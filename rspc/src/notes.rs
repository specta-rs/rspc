//! This is a temporary module i'm using to store notes until v1.
//!
//! ## Rust language limitations:
//!  - Support for Serde zero-copy deserialization
//!    - We need a way to express `where F: Fn(..., I<'_>), I<'a>: Input<'a>` which to my best knowledge is impossible.
//!
//! ## More work needed:
//!  - Yank all v1 releases once 0.3.0 is out
//!  - new Rust diagnostic stuff on all the traits
//!  - handling of result types is less efficient that it could be
//!     - If it can only return one type can be do the downcast/deserialization magic internally.
//!  - `ResolverOutput` trait oddities
//!     - `rspc::Stream` within a `rspc::Stream` will panic at runtime
//!     - Am I happy with `Output::into_procedure_stream`? It's low key cringe but it might be fine.
//!  - `ProcedureInput` vs `ResolverInput` typesafety
//!    - You can implement forget to implement `ProcedureInput` for an `ResolverInput` type.
//!    - For non-serde types really `ResolverInput` and `ProcedureInput` are the same, can we express that? Probs not without specialization and markers.
//!  - Are the restricted return types fine
//!     - We have heavily restricted return types to reduce marker traits and help the compiler with errors.
//!     - Was this a good move?
//!
