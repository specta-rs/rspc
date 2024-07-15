use std::{error, marker::PhantomData};

use rspc::procedure::{Procedure, ProcedureBuilder, ResolverInput, ResolverOutput};
use thiserror::Error;

mod chat;

#[derive(Debug, Error)]
pub enum Error {}

pub type Context = ();

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

pub fn mount() -> rspc::Router {
    rspc::Router::builder().procedure("version", {
        <BaseProcedure>::builder().query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
    })
}

// TODO: Unit test for exporting bindings
