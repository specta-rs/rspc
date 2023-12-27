//! rspc core

// TODO: Clippy lints

mod executor;
#[doc(hidden)]
pub mod internal;
mod router;
mod serializer;
mod task;

// pub trait Body = Stream<Item = Vec<u8>>; // TODO: Should this allow returning `Value`

pub use executor::{Executor, Procedure};
pub use router::Router;
pub use task::Task;

// // TODO: How can `ctx_fn` get stuff from request context or Tauri handle???
// pub fn todo_router<R: Router>(router: R, ctx_fn: impl Fn() -> R::Ctx) {
//     let executor = router.build();

//     // executor.execute();
// }
