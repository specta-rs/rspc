use rspc::{
    internal::BuiltProcedureBuilder,
    // TODO: All of these following types should probs be in `rspc::internal`!
    DoubleArgMarker,
    FutureMarker,
    RequestResolver,
    RequestResult,
    ResultMarker,
};
use serde::de::DeserializeOwned;
use specta::Type;

use crate::Object;

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
    // TODO: rspc global store accessible here

    // TODO: Mount into store and error on duplicate type name

    println!("STARTUP {}", TResolver::Data::type_name());

    BuiltProcedureBuilder {
        resolver: move |ctx, arg| {
            let val = builder.resolver.exec(ctx, arg);

            async {
                let x = val?.exec().await.unwrap();
                Ok(x.normalize())
            }
        },
    }
}
