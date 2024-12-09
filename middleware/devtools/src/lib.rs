//! rspc-devtools: Devtools for rspc applications
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

// http://[::]:4000/rspc/~rspc.devtools.meta
// http://[::]:4000/rspc/~rspc.devtools.history

mod types;

use std::{
    any::Any,
    future,
    sync::{Arc, Mutex, PoisonError},
};

use futures::stream;
use rspc_core::{Procedure, ProcedureStream, Procedures};
use types::{Metadata, ProcedureMetadata};

pub fn mount<TCtx: 'static>(
    procedures: impl Into<Procedures<TCtx>>,
    types: &impl Any,
) -> Procedures<TCtx> {
    let procedures = procedures.into();
    let meta = Metadata {
        crate_name: env!("CARGO_PKG_NAME"),
        crate_version: env!("CARGO_PKG_VERSION"),
        rspc_version: env!("CARGO_PKG_VERSION"),
        procedures: procedures
            .iter()
            .map(|(name, _)| (name.to_string(), ProcedureMetadata {}))
            .collect(),
    };
    let history = Arc::new(Mutex::new(Vec::new())); // TODO: Stream to clients instead of storing in memory

    let mut procedures = procedures
        .into_iter()
        .map(|(name, procedure)| {
            let history = history.clone();

            (
                name.clone(),
                Procedure::new(move |ctx, input| {
                    let start = std::time::Instant::now();
                    let result = procedure.exec(ctx, input);
                    history
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push((name.to_string(), format!("{:?}", start.elapsed())));
                    result
                }),
            )
        })
        .collect::<Procedures<_>>();

    procedures.insert(
        "~rspc.devtools.meta".into(),
        Procedure::new(move |ctx, input| {
            let value = Ok(meta.clone());
            ProcedureStream::from_stream(stream::once(future::ready(value)))
        }),
    );
    procedures.insert(
        "~rspc.devtools.history".into(),
        Procedure::new({
            let history = history.clone();
            move |ctx, input| {
                let value = Ok(history
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone());
                ProcedureStream::from_stream(stream::once(future::ready(value)))
            }
        }),
    );

    procedures
}
