use std::marker::PhantomData;

use crate::{Procedure, Resolver};

pub struct Router<TCtx = ()> {
    queries: Vec<Procedure<TCtx>>,
    mutations: Vec<Procedure<TCtx>>,
    subscriptions: Vec<Procedure<TCtx>>,
}

impl<TCtx> Router<TCtx> {
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
            mutations: Vec::new(),
            subscriptions: Vec::new(),
        }
    }

    pub fn middleware(mut self) -> Router<TCtx> {
        // TODO

        self
    }

    pub fn query<TResult, TMarker>(
        mut self,
        name: &'static str,
        resolver: impl Resolver<TCtx, TResult, TMarker> + 'static,
    ) -> Self {
        // TODO: Run through all middleware

        self.queries.push(Procedure {
            name: name.into(),
            exec: Box::new(move |ctx, arg| resolver.exec(ctx, arg)),
        });
        self
    }
}
