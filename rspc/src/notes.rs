//! This is a temporary module i'm using to store notes until v1.
//!
//! ## Rust language limitations:
//!  - Support for Serde zero-copy deserialization
//!    - We need a way to express `where F: Fn(..., I<'_>), I<'a>: Input<'a>` which to my best knowledge is impossible.
//!
//! ## More work needed:
//!  - handling of result types is less efficient that it could be
//!  - `Output` trait oddities
//!     - `rspc::Stream` within a `rspc::Stream` will panic at runtime
//!     - Am I happy with `Output::into_procedure_stream`? It's low key cringe but it might be fine.
//!  - `Argument` vs `Input` typesafety
//!    - You can implement forget to implement `Argument` for an `Input` type.
//!    - For non-serde types really `Input` and `Argument` are the same, can we express that? Probs not without specialization and markers.
//!  - Are the restricted return types fine
//!     - We have heavily restricted return types to reduce marker traits and help the compiler with errors.
//!     - Was this a good move?
//!
