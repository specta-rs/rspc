//! rspc-invalidation: Real-time invalidation support for rspc

use rspc::middleware::Middleware;

/// TODO
pub fn invalidation<TError, TThisCtx, TThisInput, TThisResult>(
    handler: impl Fn(TThisInput) -> bool,
) -> Middleware<TError, TThisCtx, TThisInput, TThisResult>
where
    TError: 'static,
    TThisCtx: Send + 'static,
    TThisInput: Send + 'static,
    TThisResult: Send + 'static,
{
    Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await })
}

// TODO: Attach the subscription for updates
// TODO: How do we invoke the procedures from the `invalidation.subscribe` procedure?

// TODO: What will the frontend component look like???
