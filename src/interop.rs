use std::borrow::Cow;

use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use rspc_core::{ProcedureStream, ResolverError};
use serde_json::Value;

use crate::{
    internal::{jsonrpc::JsonRPCError, Layer, ProcedureKind, RequestContext, ValueOrStream},
    Procedure2, Router, Router2,
};

pub fn legacy_to_modern<TCtx>(mut router: Router<TCtx>) -> Router2<TCtx> {
    let mut r = Router2::new();

    let bridged_procedures = router
        .queries
        .store
        .into_iter()
        .map(|v| (ProcedureKind::Query, v))
        .chain(
            router
                .mutations
                .store
                .into_iter()
                .map(|v| (ProcedureKind::Mutation, v)),
        )
        .chain(
            router
                .subscriptions
                .store
                .into_iter()
                .map(|v| (ProcedureKind::Subscription, v)),
        )
        .map(|(kind, (key, p))| {
            (
                key.split(".")
                    .map(|s| s.to_string().into())
                    .collect::<Vec<Cow<'static, str>>>(),
                Procedure2 {
                    kind,
                    input: p.ty.arg_ty,
                    result: p.ty.result_ty,
                    error: specta::datatype::DataType::Unknown,
                    inner: layer_to_procedure(key, kind, p.exec),
                },
            )
        });

    for (key, procedure) in bridged_procedures {
        if r.interop_procedures()
            .insert(key.clone(), procedure)
            .is_some()
        {
            panic!("Attempted to mount '{key:?}' multiple times. Note: rspc no longer supports different operations (query/mutation/subscription) with overlapping names.")
        }
    }

    r.interop_types().extend(&mut router.type_map);
    r
}

fn layer_to_procedure<TCtx: 'static>(
    path: String,
    kind: ProcedureKind,
    value: Box<dyn Layer<TCtx>>,
) -> rspc_core::Procedure<TCtx> {
    rspc_core::Procedure::new(move |ctx, input| {
        let input: Value = input.deserialize().unwrap(); // TODO: Error handling
        let result = value
            .call(
                ctx,
                input,
                RequestContext {
                    kind: kind.clone(),
                    path: path.clone(),
                },
            )
            .unwrap(); // TODO: Error handling

        ProcedureStream::from_stream(
            async move {
                let result = result.into_value_or_stream().await.unwrap(); // TODO: Error handling

                match result {
                    ValueOrStream::Value(value) => stream::once(async { Ok(value) }).boxed(),
                    ValueOrStream::Stream(s) => s
                        .map_err(|err| {
                            let err = JsonRPCError::from(err);
                            ResolverError::new(
                                err.code.try_into().unwrap(),
                                err,
                                None::<std::io::Error>,
                            )
                        })
                        .boxed(),
                }
            }
            .flatten_stream(),
        )
    })
}
