//! rspc-invalidation: Real-time invalidation support for rspc
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

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
