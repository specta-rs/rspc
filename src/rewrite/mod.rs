pub mod middleware;
pub mod procedure;

mod error;
mod infallible;
mod router;
mod state;
mod stream;

pub use error::Error;
pub use infallible::Infallible;
pub use router::{BuiltRouter, Router};
pub use state::State;
pub use stream::Stream;

#[doc(hidden)]
pub mod internal {
    // To make versioning easier we reexport it so libraries such as `rspc_axum` don't need a direct dependency on `specta`.
    pub use serde::Serialize;
    pub use specta::{DataType, Type, TypeMap}; // TODO: Why does rspc_axum care again?
}
