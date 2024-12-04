use std::{borrow::Cow, collections::BTreeMap};

use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use rspc_core::{ProcedureStream, ResolverError};
use serde_json::Value;
use specta::{datatype::DataType, NamedType, SpectaID, Type};

use crate::{
    internal::{jsonrpc::JsonRPCError, Layer, ProcedureKind, RequestContext, ValueOrStream},
    Router, Router2,
};

pub fn legacy_to_modern<TCtx>(mut router: Router<TCtx>) -> Router2<TCtx> {
    let mut r = Router2::new();

    let (types, layers): (Vec<_>, Vec<_>) = router
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
        .map(|(ty, (key, p))| {
            (
                (
                    key.clone().into(),
                    literal_object(
                        "".into(),
                        None,
                        vec![
                            ("input".into(), p.ty.arg_ty),
                            ("result".into(), p.ty.result_ty),
                        ]
                        .into_iter(),
                    ),
                ),
                (key, ty, p.exec),
            )
        })
        .unzip();

    for (key, ty, exec) in layers {
        if r.interop_procedures()
            .insert(key.clone(), layer_to_procedure(key.clone(), ty, exec))
            .is_some()
        {
            panic!("Attempted to mount '{key}' multiple times. Note: rspc no longer supports different operations (query/mutation/subscription) with overlapping names.")
        }
    }

    {
        #[derive(Type)]
        struct Procedures;

        let s = literal_object(
            "Procedures".into(),
            Some(Procedures::sid()),
            types.into_iter(),
        );
        let mut ndt = Procedures::definition_named_data_type(&mut r.interop_types());
        ndt.inner = s.into();
        r.interop_types().insert(Procedures::sid(), ndt);
    }

    r.interop_types().extend(&mut router.type_map);

    r
}

// TODO: Probally using `DataTypeFrom` stuff cause we shouldn't be using `specta::internal`
fn literal_object(
    name: Cow<'static, str>,
    sid: Option<SpectaID>,
    fields: impl Iterator<Item = (Cow<'static, str>, DataType)>,
) -> DataType {
    specta::internal::construct::r#struct(
        name,
        sid,
        Default::default(),
        specta::internal::construct::struct_named(
            fields
                .into_iter()
                .map(|(name, ty)| {
                    (
                        name.into(),
                        specta::internal::construct::field(false, false, None, "".into(), Some(ty)),
                    )
                })
                .collect(),
            None,
        ),
    )
    .into()
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
