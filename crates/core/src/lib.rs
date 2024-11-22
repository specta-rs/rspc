//! rspc-core
//!
//! TODO: Describe all the types and why the split?
//! TODO: This is kinda like `tower::Service`
// TODO: Crate icon and stuff

// TODO: Solve layer:
// - Crate documentation
// - `DynInput<'a, 'de>` should really be &'a Input<'de>` but that's hard.
// - `DynInput` errors:
//    - store `type_name` for better errors.
//    - Deserializer error

mod dyn_input;
mod error;
mod procedure;
mod stream;

pub use dyn_input::DynInput;
pub use procedure::Procedure;
pub use stream::ProcedureStream;

// TODO: Should `Procedure` hold types? It prevents them from being removed at runtime.

// TODO: Async or sync procedures??
// TODO: Single-threaded async support

// TODO: Result types
// TODO: Typesafe error handling

// TODO: non-'static TypeId would prevent the need for `Argument` vs `Input` because you could parse down `&dyn erased_serde::Deserializer`.

// TODO: The two `exec` methods is survivable by the problem is that we have `Input` and
// need to go from it to `TInput` within the erased procedure, either via Deserialize or downcast.
//
// We can avoid doing this in `rspc_core` but it's still a problem `rspc` needs to deal with.

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
