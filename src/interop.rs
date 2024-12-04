use std::{borrow::Cow, collections::BTreeMap};

use futures::{stream, FutureExt, Stream, StreamExt, TryStreamExt};
use rspc_core::{ProcedureStream, ResolverError};
use serde_json::Value;
use specta::{
    datatype::{DataType, EnumRepr, EnumVariant, LiteralType},
    NamedType, Type,
};

use crate::{
    internal::{jsonrpc::JsonRPCError, Layer, ProcedureKind, RequestContext, ValueOrStream},
    router::literal_object,
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
                    setup: Default::default(),
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
        let result = input
            .deserialize::<Value>()
            .map_err(Into::into)
            .and_then(|input| {
                value
                    .call(
                        ctx,
                        input,
                        RequestContext {
                            kind: kind.clone(),
                            path: path.clone(),
                        },
                    )
                    .map_err(|err| {
                        let err: crate::legacy::Error = err.into();
                        ResolverError::new(
                            err.code.to_status_code(),
                            (), /* typesafe errors aren't supported in legacy router */
                            Some(rspc_core::LegacyErrorInterop(err.message)),
                        )
                    })
            });

        match result {
            Ok(result) => ProcedureStream::from_future_stream(async move {
                match result.into_value_or_stream().await {
                    Ok(ValueOrStream::Value(value)) => {
                        Ok(stream::once(async { Ok(value) }).boxed())
                    }
                    Ok(ValueOrStream::Stream(s)) => Ok(s
                        .map_err(|err| {
                            let err = JsonRPCError::from(err);
                            ResolverError::new(
                                err.code.try_into().unwrap_or(500),
                                err,
                                None::<std::io::Error>,
                            )
                        })
                        .boxed()),
                    Err(err) => {
                        let err: crate::legacy::Error = err.into();
                        let err =
                            ResolverError::new(err.code.to_status_code(), err.message, err.cause);
                        // stream::once(async { Err(err) }).boxed()
                        Err(err)
                    }
                }
            }),
            Err(err) => ProcedureStream::from_value(Err::<(), _>(err)),
        }
    })
}

fn map_method<TCtx>(
    kind: ProcedureKind,
    p: &BTreeMap<Vec<Cow<'static, str>>, Procedure2<TCtx>>,
) -> Vec<(Cow<'static, str>, EnumVariant)> {
    p.iter()
        .filter(|(_, p)| p.kind() == kind)
        .map(|(key, p)| {
            let key = key.join(".").to_string();
            (
                key.clone().into(),
                specta::internal::construct::enum_variant(
                    false,
                    None,
                    "".into(),
                    specta::internal::construct::enum_variant_unnamed(vec![
                        specta::internal::construct::field(
                            false,
                            false,
                            None,
                            "".into(),
                            Some(literal_object(
                                "".into(),
                                None,
                                vec![
                                    ("key".into(), LiteralType::String(key.clone()).into()),
                                    ("input".into(), p.input.clone()),
                                    ("result".into(), p.result.clone()),
                                ]
                                .into_iter(),
                            )),
                        ),
                    ]),
                ),
            )
        })
        .collect::<Vec<_>>()
}

// TODO: Remove this block with the interop system
pub(crate) fn construct_legacy_bindings_type<TCtx>(
    p: &BTreeMap<Vec<Cow<'static, str>>, Procedure2<TCtx>>,
) -> Vec<(Cow<'static, str>, DataType)> {
    #[derive(Type)]
    struct Queries;
    #[derive(Type)]
    struct Mutations;
    #[derive(Type)]
    struct Subscriptions;

    vec![
        (
            "queries".into(),
            specta::internal::construct::r#enum(
                "Queries".into(),
                Queries::sid(),
                EnumRepr::Untagged,
                false,
                Default::default(),
                map_method(ProcedureKind::Query, p),
            )
            .into(),
        ),
        (
            "mutations".into(),
            specta::internal::construct::r#enum(
                "Mutations".into(),
                Mutations::sid(),
                EnumRepr::Untagged,
                false,
                Default::default(),
                map_method(ProcedureKind::Mutation, p),
            )
            .into(),
        ),
        (
            "subscriptions".into(),
            specta::internal::construct::r#enum(
                "Subscriptions".into(),
                Subscriptions::sid(),
                EnumRepr::Untagged,
                false,
                Default::default(),
                map_method(ProcedureKind::Subscription, p),
            )
            .into(),
        ),
    ]
}
