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
    sync::{Arc, Mutex, PoisonError},
};

use rspc_core::{Procedure, ProcedureStream, Procedures};
use types::{Metadata, ProcedureMetadata};

pub fn mount<TCtx: 'static>(
    routes: impl Into<Procedures<TCtx>>,
    types: &impl Any,
) -> impl Into<Procedures<TCtx>> {
    let procedures = routes.into();
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
                    let result = procedure.exec_with_dyn_input(ctx, input);
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
        Procedure::new(move |ctx, input| ProcedureStream::from_value(Ok(meta.clone()))),
    );
    procedures.insert(
        "~rspc.devtools.history".into(),
        Procedure::new({
            let history = history.clone();
            move |ctx, input| {
                ProcedureStream::from_value(Ok(history
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone()))
            }
        }),
    );

    procedures
}
