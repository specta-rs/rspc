use std::{any::TypeId, collections::HashMap};

use rspc::internal::{
    BuiltProcedureBuilder, DoubleArgMarker, FutureMarker, RequestResolver, RequestResult,
    ResultMarker,
};
use serde::de::DeserializeOwned;
use specta::Type;

use crate::Object;

#[derive(Default)]
pub struct NormiContext {
    types: HashMap<&'static str, TypeId>,
}

// TODO: Convert this in a macro in rspc maybe???
pub fn typed<TLayerCtx, TResolver, TArg, TResolverMarker, TResultMarker, TIncomingResult>(
    builder: BuiltProcedureBuilder<TResolver>,
) -> BuiltProcedureBuilder<
    impl RequestResolver<
        TLayerCtx,
        DoubleArgMarker<TArg, FutureMarker<ResultMarker>>,
        FutureMarker<ResultMarker>,
        Arg = TArg,
        Data = <TIncomingResult::Data as Object>::NormalizedResult,
    >,
>
where
    TLayerCtx: Send + Sync + 'static,
    TResolver: RequestResolver<
        TLayerCtx,
        TResolverMarker,
        TResultMarker,
        Arg = TArg,
        Result = TIncomingResult,
    >,
    TResolver::Data: Object,
    TIncomingResult: RequestResult<TResultMarker>,
    TIncomingResult::Data: Object,
    // <<TResolver as RequestResolver<TLayerCtx, TResolverMarker, TResultMarker>>::Result as RequestResult<TResultMarker>>::Data: Object,
    TArg: Type + DeserializeOwned,
{
    {
        let mut data = builder.data.as_ref().unwrap().write().unwrap();

        let ctx = data
            .entry(TypeId::of::<NormiContext>())
            .or_insert_with(|| Box::new(NormiContext::default()) as Box<dyn std::any::Any>)
            .downcast_mut::<NormiContext>()
            .unwrap();

        let expected_tid = ctx
            .types
            .entry(TResolver::Data::type_name())
            .or_insert_with(|| TypeId::of::<TResolver::Data>());
        if *expected_tid != TypeId::of::<TResolver::Data>() {
            panic!("A types with typeid '{:?}' and '{:?}' were both mounted with an identical type name '{}'. The type name must be unique!", expected_tid, TypeId::of::<TResolver::Data>(), TResolver::Data::type_name());
        }
    }

    BuiltProcedureBuilder {
        data: builder.data,
        resolver: move |ctx, arg| {
            let val = builder.resolver.exec(ctx, arg);

            async {
                let x = val?.exec().await.unwrap();
                Ok(x.normalize().unwrap())
            }
        },
    }
}
