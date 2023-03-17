// pub trait Procedure {
//     // type CtxIn;
//     // type CtxOut;

//     // type Future;
//     // type Result;
//     // type Error;

//     // type Meta;
// }

use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    internal::{BuiltProcedureBuilder, ResolverLayer, UnbuiltProcedureBuilder},
    typedef, ExecError, RequestLayer,
};

use super::{IntoProcedure, IntoProcedureCtx};

// TODO: Check metadata stores on this so plugins can extend it to do cool stuff
// TODO: Logical order for these generics cause right now it's random
pub struct AlphaProcedure<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, TMeta>(
    TBuilder,
    PhantomData<(TLayerCtx, TResolver, TArg, TResult, TResultMarker, TMeta)>,
)
where
    TLayerCtx: Send + Sync + 'static,
    TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: DeserializeOwned + Type,
    TResult: RequestLayer<TResultMarker>,
    // TODO: Split this apart and store into `Self` + should it be `FnOnce` if it stays?
    TBuilder: Fn(UnbuiltProcedureBuilder<TLayerCtx, TResolver>) -> BuiltProcedureBuilder<TResolver>;

impl<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, TMeta>
    AlphaProcedure<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, TMeta>
where
    TLayerCtx: Send + Sync + 'static,
    TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: DeserializeOwned + Type,
    TResult: RequestLayer<TResultMarker>,

    TBuilder: Fn(UnbuiltProcedureBuilder<TLayerCtx, TResolver>) -> BuiltProcedureBuilder<TResolver>,
{
    pub(crate) fn new(builder: TBuilder) -> Self {
        Self(builder, PhantomData)
    }
}

// TODO: Allowing a plugin to require a specific type for `TMeta`, idk???
impl<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder>
    AlphaProcedure<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, ()>
where
    TLayerCtx: Send + Sync + 'static,
    TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: DeserializeOwned + Type,
    TResult: RequestLayer<TResultMarker>,
    TBuilder: Fn(UnbuiltProcedureBuilder<TLayerCtx, TResolver>) -> BuiltProcedureBuilder<TResolver>,
{
    // TODO
    pub fn meta<TMeta: 'static>(
        self,
        meta: TMeta,
    ) -> AlphaProcedure<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, TMeta> {
        // TODO: Store `meta` so it can be used
        AlphaProcedure(self.0, PhantomData)
    }
}

impl<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, TMeta> IntoProcedure<TLayerCtx>
    for AlphaProcedure<TLayerCtx, TResolver, TArg, TResult, TResultMarker, TBuilder, TMeta>
where
    TLayerCtx: Send + Sync + 'static,
    TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: DeserializeOwned + Type + 'static,
    TResult: RequestLayer<TResultMarker> + 'static,
    TResultMarker: 'static,
    TBuilder: Fn(UnbuiltProcedureBuilder<TLayerCtx, TResolver>) -> BuiltProcedureBuilder<TResolver>
        + 'static,
    TMeta: 'static,
{
    fn build(&mut self, key: Cow<'static, str>, ctx: &mut IntoProcedureCtx<'_, TLayerCtx>) {
        let resolver = self.0(UnbuiltProcedureBuilder::default()).resolver;
        ctx.queries.append(
            key.into(),
            // self.middleware.build(ResolverLayer {
            //     func: move |ctx, input, _| {
            //         resolver.exec(
            //             ctx,
            //             serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
            //         )
            //     },
            //     phantom: PhantomData,
            // }),
            Box::new(ResolverLayer {
                // TODO: Still gotta apply middleware here
                func: move |ctx, input, _| {
                    resolver(
                        ctx,
                        serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
                    )
                    .into_layer_result()
                },
                phantom: PhantomData,
            }),
            typedef::<TArg, TResult::Result>(ctx.ty_store),
        );
    }
}

pub trait ProcedureBuilder: FnOnce() -> () {}
impl<T> ProcedureBuilder for T where T: FnOnce() -> () {}
