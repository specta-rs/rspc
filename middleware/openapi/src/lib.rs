//! rspc-openapi: OpenAPI support for rspc
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

use std::{borrow::Cow, collections::HashMap};

use rspc::middleware::Middleware;

#[derive(Default)]
pub struct OpenAPIState(HashMap<Cow<'static, str>, ()>);

// TODO: Configure other OpenAPI stuff like auth

// TODO: Make convert this into a builder like: Endpoint::get("/todo").some_other_stuff().build()
pub fn openapi<TError, TThisCtx, TThisInput, TThisResult>(
    // method: Method,
    path: impl Into<Cow<'static, str>>,
) -> Middleware<TError, TThisCtx, TThisInput, TThisResult>
where
    TError: 'static,
    TThisCtx: Send + 'static,
    TThisInput: Send + 'static,
    TThisResult: Send + 'static,
{
    let path = path.into();
    Middleware::new(|ctx, input, next| async move {
        let _result = next.exec(ctx, input).await;
        _result
    })
    .setup(|state, meta| {
        state
            .get_mut_or_init::<OpenAPIState>(Default::default)
            .0
            .insert(path, ());
    })
}

// TODO: Convert into API endpoint
