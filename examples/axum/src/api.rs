use std::{error, marker::PhantomData, path::PathBuf, sync::Arc};

use rspc::{
    procedure::{Procedure, ProcedureBuilder, ProcedureKind, ResolverInput, ResolverOutput},
    Infallible,
};
use specta_typescript::Typescript;
use specta_util::TypeCollection;
use thiserror::Error;

pub(crate) mod chat;
pub(crate) mod invalidation;
pub(crate) mod store;

#[derive(Debug, Error)]
pub enum Error {}

// `Clone` is only required for usage with Websockets
#[derive(Clone)]
pub struct Context {
    // For this example we nest context's for each example.
    // In the real-world you don't need to do this, we do this so the examples are more self-contained.
    pub chat: chat::Ctx,
    pub invalidation: Arc<invalidation::Ctx>,
}

pub type Router = rspc::Router<Context>;

pub struct BaseProcedure<TErr = Error>(PhantomData<TErr>);

impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>() -> ProcedureBuilder<TErr, Context, Context, TInput, TResult>
    where
        TErr: error::Error + Send + 'static,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure::builder() // You add default middleware here
    }
}

pub fn mount() -> Router {
    Router::new()
        .procedure("version", {
            <BaseProcedure>::builder().query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
        })
        .merge("chat", chat::mount())
        .merge("store", store::mount())
        // TODO: I dislike this API
        .ext({
            let mut types = TypeCollection::default();
            types.register::<Infallible>();
            types
        })
        .export_to(
            Typescript::default(),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
        )
}

#[cfg(test)]
#[test]
fn export_rspc_router() {
    mount().build().unwrap();
}
