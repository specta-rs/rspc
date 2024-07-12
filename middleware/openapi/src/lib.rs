//! rspc-openapi: OpenAPI support for rspc

use std::{borrow::Cow, collections::HashMap};

use rspc::middleware::Middleware;

#[derive(Default)]
pub struct OpenAPIState(HashMap<Cow<'static, str>, ()>);

// TODO: Configure other OpenAPI stuff like auth

pub fn openapi<TError, TThisCtx, TThisInput, TThisResult>(
    // method: Method,
    path: impl Into<Cow<'static, str>>,
) -> Middleware<TError, TThisCtx, TThisInput, TThisResult> {
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
