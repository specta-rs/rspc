//! TODO: Use `axum_core` not `axum`

use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, Request},
};
use rspc_core::{Procedure, ProcedureStream};
use serde::Deserializer;

// TODO: rename?
pub struct AxumRequest {
    req: Request,
}

impl AxumRequest {
    pub fn deserialize<T>(self, exec: impl FnOnce(&[u8]) -> T) -> T {
        let hint = self.req.body().size_hint();
        let has_body = hint.lower() != 0 || hint.upper() != Some(0);

        // TODO: Matching on incoming method???

        // let mut bytes = None;
        // let input = if !has_body {
        //     ExecuteInput::Query(req.uri().query().unwrap_or_default())
        // } else {
        //     // TODO: bring this back
        //     // if !json_content_type(req.headers()) {
        //     //     let err: ProcedureError = rspc_core::DeserializeError::custom(
        //     //         "Client did not set correct valid 'Content-Type' header",
        //     //     )
        //     //     .into();
        //     //     let buf = serde_json::to_vec(&err).unwrap(); // TODO

        //     //     return (
        //     //         StatusCode::BAD_REQUEST,
        //     //         [(header::CONTENT_TYPE, "application/json")],
        //     //         Body::from(buf),
        //     //     )
        //     //         .into_response();
        //     // }

        //     // TODO: Error handling
        //     bytes = Some(Bytes::from_request(req, &()).await.unwrap());
        //     ExecuteInput::Body(
        //         bytes.as_ref().expect("assigned on previous line"),
        //     )
        // };

        // let (status, stream) =
        //     rspc_http::execute(&procedure, input, || ctx_fn()).await;

        exec(b"null")
    }
}

#[async_trait]
impl<S> FromRequest<S> for AxumRequest {
    type Rejection = (); // TODO: What should this be?

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self { req })
    }
}
