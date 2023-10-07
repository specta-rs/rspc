use rspc::RouterBuilder;
use serde::Serialize;
use specta::Type;

use crate::R;

#[derive(thiserror::Error, serde::Serialize, specta::Type, Debug)]
#[error("{0}")]
struct Error(&'static str);

#[derive(thiserror::Error, Serialize, Type, Debug)]
pub enum MyCustomError {
    #[error("I am broke")]
    IAmBroke,
}

// We merge this router into the main router in `main.rs`.
// This router shows how to do error handling
pub fn mount() -> RouterBuilder<()> {
    R.router()
        .procedure("ok", R.query(|_, _args: ()| Ok("Hello World")))
        .procedure(
            "err",
            R.error::<Error>()
                .query(|_, _args: ()| Err::<String, _>(Error("This is a custom error!"))),
        )
        .procedure(
            "customErr",
            R.error::<MyCustomError>()
                .query(|_, _args: ()| Err::<String, _>(MyCustomError::IAmBroke)),
        )
        .procedure(
            "asyncCustomError",
            R.error::<MyCustomError>()
                .mutation(|_, _args: ()| async move { Err::<String, _>(MyCustomError::IAmBroke) }),
        )
}
