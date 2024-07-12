//! rspc-tracing: Tracing support for rspc

use rspc::middleware::Middleware;

pub fn logging<TError, TThisCtx, TThisInput, TThisResult>(
) -> Middleware<TError, TThisCtx, TThisInput, TThisResult> {
    Middleware::new(|ctx, input, next| async move {
        let _result = next.exec(ctx, input).await;
        _result
    })
}
