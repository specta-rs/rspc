//! rspc-core
//!
//! TODO: Describe all the types and why the split?
//! TODO: This is kinda like `tower::Service`
//! TODO: Why this crate doesn't depend on Specta.
//! TODO: Discuss the traits that need to be layered on for this to be useful.
//! TODO: Discuss how middleware don't exist here.
// TODO: Crate icon and stuff

// - Returning non-Serialize types (Eg. `File`) via `ProcedureStream`.
//
// - Rename `DynInput` to `DynValue` maybe???
// - `ProcedureStream` to `impl futures::Stream` adapter.
// - `ProcedureStream::poll_next` - Keep or remove???
// - `Send` + `Sync` and the issues with single-threaded async runtimes
// - `DynInput<'a, 'de>` should really be &'a Input<'de>` but that's hard.
// - Finish `Debug` impls
// - Crate documentation

mod dyn_input;
mod error;
mod procedure;
mod stream;

pub use dyn_input::DynInput;
pub use error::{DeserializeError, DowncastError, ProcedureError, ResolverError};
pub use procedure::Procedure;
pub use stream::ProcedureStream;

pub type Procedures<TCtx> =
    std::collections::BTreeMap<Vec<std::borrow::Cow<'static, str>>, Procedure<TCtx>>;

// TODO: Remove this once we remove the legacy executor.
#[doc(hidden)]
#[derive(Clone)]
pub struct LegacyErrorInterop(pub String);
impl std::fmt::Debug for LegacyErrorInterop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LegacyErrorInterop({})", self.0)
    }
}
impl std::fmt::Display for LegacyErrorInterop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LegacyErrorInterop({})", self.0)
    }
}
impl std::error::Error for LegacyErrorInterop {}

// TODO: The naming is horid.
// Low-level concerns:
// - `Procedure` - Holds the handler (and probably type information)
//     - `Procedure::new(|ctx, input| { ... })`
//     - `Procedure::exec_with_deserializer(ctx, input)`
//     - `Procedure::exec_with_value(ctx, input)`
//
// - `Input` (prev. `ProcedureExecInput`) - Holds a mutable reference to either a value or a deserializer.
//     - Used within function given to `Procedure::new`.
//     - Eg. `|ctx, input| input.deserialize::<T>().unwrap()` or
//           `|ctx, input| input.value::<T>().unwrap()`
//
//     - This type exists so we can slap the generic methods on the dyn-safe types it wraps.
//
// - `Output` - What the resolver function can return.
//     - Basically the same as `Input` but reverse direction.
//
// - `ResolverOutput` - What `exec` can return. Either a `Serializer` or `T` (Eg. `File`).
//     - Basically the same as `ProcedureInput` but reverse direction.
//
// - `Stream`/`ProcedureType` - Self explanatory.
//
// High-level concerns:
// - `ProcedureBuilder` - The thing that converts the high-level resolver fn to the low-level one.
//      - This is where middleware are introduced. The low-level system doesn't know about them.
//
// - `ResolverInput` - A resolver takes `TInput`. This trait allows us to go `rspc_core::Input` to `TInput`.
//      - Is implemented for `T: DeserializeOwned` and any custom type (Eg. `File`).
//
//      - It would be nice if this could replace by `Input::from_value` and `Input::from_deserializer`.
//        However, due to `erased_serde` this is basically impossible.
//
// - `ProcedureInput` - `exec` takes a `T`. This trait allows us to go `T` to `rspc_core::Input`.
//      - Is implemented for `T: Deserializer<'de>` and any custom type (Eg. `File`).
//      - This would effectively dispatch to `exec_with_deserializer` or `exec_with_value`.
//
//      - We could avoid this method an eat the DX (2 methods) and
//        loss of typesafety (any `'static` can be parsed in even if missing `Argument1` impl).
//
//      - Kinda has to end up in `rspc_core` so it can be used by `rspc_axum` for `File`.
//
// Notes:
// - Due to how erased_serde works `input.deserialize::<T>()` works with `#[serde(borrow)]`.
//   However, given we can't `impl for<'a> Fn(_, impl Argument<'a>)` it won't work for the high-level API.
//

// A decent cause of the bloat is because `T` (Eg. `File`), `Deserializer` and `Deserialize` are all different. You end up with a `ResolverInput` trait which is `Deserialize` + `T` , a `ProcedureInput` trait which is `Deserializer` + `T` and then `ExecInput` which is the dyn-safe output of  `ProcedureInput` and is given into `ResolverInput` so it can decode it back to the value the user expects. Then you basically copy the same thing for the output value. I think it might be worth replacing `ProcedureInput` with `Procedure::exec_with_deserializer` and `Procedure::exec_with_value` but i'm not sure we could get away with doing the same thing for `ResolverInput` because that would mean requiring two forms of queries/mutations/subscriptions in the high-level API. That being said `ResolverInput` could probably be broken out of the procedure primitive.

// TODO: The new system doesn't allocate Serde related `Error`'s and Serde return values, pog.
