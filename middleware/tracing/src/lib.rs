//! rspc-tracing: Tracing support for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

use std::{fmt, marker::PhantomData};

use rspc::modern::middleware::Middleware;
use tracing::info;

mod traceable;

pub use traceable::{DebugMarker, StreamMarker, Traceable};
use tracing_futures::Instrument;

// TODO: Support for Prometheus metrics and structured logging

/// TODO
pub fn tracing<TError, TCtx, TInput, TResult, M>() -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: fmt::Debug + Send + 'static,
    TCtx: Send + 'static,
    TInput: fmt::Debug + Send + 'static,
    TResult: Traceable<M> + Send + 'static,
{
    Middleware::new(|ctx, input, next| {
        let span = tracing::info_span!(
            "",
            "{} {}",
            next.meta().kind().to_string().to_uppercase(), // TODO: Maybe adding color?
            next.meta().name()
        );

        async move {
            let input_str = format!("{input:?}");
            let start = std::time::Instant::now();
            let result = next.exec(ctx, input).await;
            info!(
                "took {:?} with input {input_str:?} and returned {:?}",
                start.elapsed(),
                DebugWrapper(&result, PhantomData::<M>)
            );

            result
        }
        .instrument(span)
    })
}

struct DebugWrapper<'a, T: Traceable<M>, TErr, M>(&'a Result<T, TErr>, PhantomData<M>);

impl<'a, T: Traceable<M>, TErr: fmt::Debug, M> fmt::Debug for DebugWrapper<'a, T, TErr, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Ok(v) => v.fmt(f),
            Err(e) => write!(f, "{e:?}"),
        }
    }
}
