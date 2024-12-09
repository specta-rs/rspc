use std::{borrow::Cow, collections::BTreeMap, panic::Location};

use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use rspc_core::{ProcedureStream, ResolverError};
use serde_json::Value;
use specta::{
    datatype::{DataType, EnumRepr, EnumVariant, LiteralType},
    NamedType, SpectaID, Type,
};

use crate::{
    internal::{Layer, ProcedureKind, RequestContext, ValueOrStream},
    procedure::ProcedureType,
    types::TypesOrType,
    util::literal_object,
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
                    ty: ProcedureType {
                        kind,
                        input: p.ty.arg_ty,
                        output: p.ty.result_ty,
                        error: specta::datatype::DataType::Unknown,
                        // TODO: This location is obviously wrong but the legacy router has no location information.
                        // This will work properly with the new procedure syntax.
                        location: Location::caller().clone(),
                    },
                    // location: Location::caller().clone(), // TODO: This needs to actually be correct
                    inner: layer_to_procedure(key, kind, p.exec),
                },
            )
        });

    for (key, procedure) in bridged_procedures {
        if r.interop_procedures()
            .insert(key.clone(), procedure)
            .is_some()
        {
            panic!("Attempted to mount '{key:?}' multiple times.\nrspc no longer supports different operations (query/mutation/subscription) with overlapping names.")
        }
    }

    r.interop_types().extend(&mut router.type_map);
    r
}

pub(crate) fn layer_to_procedure<TCtx: 'static>(
    path: String,
    kind: ProcedureKind,
    value: Box<dyn Layer<TCtx>>,
) -> rspc_core::Procedure<TCtx> {
    rspc_core::Procedure::new(move |ctx, input| {
        let result = input.deserialize::<Value>().and_then(|input| {
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
                    .into()
                })
        });

        match result {
            Ok(result) => ProcedureStream::from_stream(
                async move {
                    match result.into_value_or_stream().await {
                        Ok(ValueOrStream::Value(value)) => {
                            stream::once(async { Ok(value) }).boxed()
                        }
                        Ok(ValueOrStream::Stream(s)) => s
                            .map_err(|err| {
                                let err = crate::legacy::Error::from(err);
                                ResolverError::new(
                                    err.code.to_status_code(),
                                    (), /* typesafe errors aren't supported in legacy router */
                                    Some(rspc_core::LegacyErrorInterop(err.message)),
                                )
                                .into()
                            })
                            .boxed(),
                        Err(err) => {
                            let err: crate::legacy::Error = err.into();
                            let err = ResolverError::new(
                                err.code.to_status_code(),
                                err.message,
                                err.cause,
                            );
                            stream::once(async { Err(err.into()) }).boxed()
                        }
                    }
                }
                .into_stream()
                .flatten(),
            ),
            Err(err) => {
                ProcedureStream::from_stream(stream::once(async move { Err::<(), _>(err) }))
            }
        }
    })
}

fn map_method(
    kind: ProcedureKind,
    p: &BTreeMap<Vec<Cow<'static, str>>, ProcedureType>,
) -> Vec<(Cow<'static, str>, EnumVariant)> {
    p.iter()
        .filter(|(_, p)| p.kind == kind)
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
                                    ("result".into(), p.output.clone()),
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
pub(crate) fn construct_legacy_bindings_type(
    map: &BTreeMap<Cow<'static, str>, TypesOrType>,
) -> Vec<(Cow<'static, str>, DataType)> {
    #[derive(Type)]
    struct Queries;
    #[derive(Type)]
    struct Mutations;
    #[derive(Type)]
    struct Subscriptions;

    let mut p = BTreeMap::new();
    for (k, v) in map {
        flatten_procedures_for_legacy(&mut p, vec![k.clone()], v.clone());
    }

    vec![
        (
            "queries".into(),
            specta::internal::construct::r#enum(
                "Queries".into(),
                Queries::sid(),
                EnumRepr::Untagged,
                false,
                Default::default(),
                map_method(ProcedureKind::Query, &p),
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
                map_method(ProcedureKind::Mutation, &p),
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
                map_method(ProcedureKind::Subscription, &p),
            )
            .into(),
        ),
    ]
}

fn flatten_procedures_for_legacy(
    p: &mut BTreeMap<Vec<Cow<'static, str>>, ProcedureType>,
    key: Vec<Cow<'static, str>>,
    item: TypesOrType,
) {
    match item {
        TypesOrType::Type(ty) => {
            p.insert(key, ty);
        }
        TypesOrType::Types(types) => {
            for (k, v) in types {
                let mut key = key.clone();
                key.push(k.clone());
                flatten_procedures_for_legacy(p, key, v);
            }
        }
    }
}
