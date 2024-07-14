//! rspc-tracing: Tracing support for rspc

use std::fmt;

use rspc::middleware::Middleware;
use tracing::info;

/// TODO
pub fn tracing<TError, TThisCtx, TThisInput, TThisResult>(
) -> Middleware<TError, TThisCtx, TThisInput, TThisResult>
where
    TError: fmt::Debug + Send + 'static,
    TThisCtx: Send + 'static,
    TThisInput: fmt::Debug + Send + 'static,
    TThisResult: fmt::Debug + Send + 'static,
{
    Middleware::new(|ctx, input, next| async move {
        let input_str = format!("{input:?}");
        let start = std::time::Instant::now();
        let result = next.exec(ctx, input).await;
        info!(
            "{} {} took {:?} with input {input_str:?} and returned {result:?}",
            next.meta().kind().to_string().to_uppercase(), // TODO: Maybe adding color?
            next.meta().name(),
            start.elapsed()
        );

        result
    })
}
