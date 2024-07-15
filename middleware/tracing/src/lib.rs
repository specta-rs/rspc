//! rspc-tracing: Tracing support for rspc

use std::{fmt, marker::PhantomData};

use rspc::middleware::Middleware;
use tracing::info;

mod traceable;

pub use traceable::{DebugMarker, StreamMarker, Traceable};

// TODO: Support for Prometheus metrics and structured logging
// TODO: Assigning a span to the entire request (function and future)

/// TODO
pub fn tracing<TError, TCtx, TInput, TResult, M>() -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: fmt::Debug + Send + 'static,
    TCtx: Send + 'static,
    TInput: fmt::Debug + Send + 'static,
    TResult: Traceable<M> + Send + 'static,
{
    Middleware::new(|ctx, input, next| async move {
        let input_str = format!("{input:?}");
        let start = std::time::Instant::now();
        let result = next.exec(ctx, input).await;
        info!(
            "{} {} took {:?} with input {input_str:?} and returned {:?}",
            next.meta().kind().to_string().to_uppercase(), // TODO: Maybe adding color?
            next.meta().name(),
            start.elapsed(),
            DebugWrapper(&result, PhantomData::<M>)
        );

        result
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
