use std::{any::TypeId, collections::HashMap};

use rspc::{
    internal::{
        BuiltProcedureBuilder, DoubleArgMarker, FutureMarker, RequestResolver, RequestResult,
        ResultMarker,
    },
    ExecError,
};
use serde::de::DeserializeOwned;
use specta::Type;

use crate::{normalise, Object};

#[derive(Default)]
pub struct NormiContext {
    types: HashMap<&'static str, TypeId>,
}

// TODO: Convert this in a macro in rspc maybe???
#[allow(clippy::panic)]
pub fn normi<TResolver, TCtx, TArg, TMarker, TResultMarker>(
    builder: BuiltProcedureBuilder<TResolver>,
) -> BuiltProcedureBuilder<
    impl RequestResolver<
        TCtx,
        DoubleArgMarker<TArg, FutureMarker<ResultMarker>>,
        FutureMarker<ResultMarker>,
        Arg = TArg,
    >,
>
where
    TResolver: RequestResolver<TCtx, TMarker, TResultMarker, Arg = TArg>,
    TResolver::Data: Object,
    <<TResolver as RequestResolver<TCtx, TMarker, TResultMarker>>::Result as RequestResult<
        TResultMarker,
    >>::Data: Object,
    TArg: Type + DeserializeOwned,
{
    {
        let mut data = builder
            .data
            .write()
            .expect("Error getting mutex lock on rspc data store!");

        let ctx = data
            .entry(TypeId::of::<NormiContext>())
            .or_insert_with(|| {
                Box::<NormiContext>::default() as Box<dyn std::any::Any + Send + Sync + 'static>
            })
            .downcast_mut::<NormiContext>()
            .expect("`NormiContext` was not of the correct type! This is almost certainly a bug in Normi.");

        let expected_tid = ctx
            .types
            .entry(TResolver::Data::type_name())
            .or_insert_with(TypeId::of::<TResolver::Data>);
        if *expected_tid != TypeId::of::<TResolver::Data>() {
            panic!("A types with typeid '{:?}' and '{:?}' were both mounted with an identical type name '{}'. The type name must be unique!", expected_tid, TypeId::of::<TResolver::Data>(), TResolver::Data::type_name());
        }
    }

    BuiltProcedureBuilder {
        name: builder.name,
        kind: builder.kind,
        typedef: builder.typedef,
        data: builder.data,
        resolver: move |ctx, arg| {
            let val = builder.resolver.exec(ctx, arg);

            async { Ok(normalise(val?.exec().await?).map_err(ExecError::SerializingResultErr)?) }
        },
    }
}
