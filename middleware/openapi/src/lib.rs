//! rspc-openapi: OpenAPI support for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

use std::{borrow::Cow, collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::Query,
    http::{request::Parts, StatusCode},
    response::Html,
    routing::{delete, get, patch, post, put},
    Json,
};
use futures::StreamExt;
use rspc::{middleware::Middleware, Procedure2, ResolverInput, Router2};
use serde_json::json;

// TODO: Properly handle inputs from query params
// TODO: Properly handle responses from query params
// TODO: Support input's coming from URL. Eg. `/todos/{id}` like tRPC-OpenAPI
// TODO: Support `application/x-www-form-urlencoded` bodies like tRPC-OpenAPI
// TODO: Probs put SwaggerUI behind a feature flag

pub struct OpenAPI {
    method: &'static str,
    path: Cow<'static, str>,
}

impl OpenAPI {
    // TODO
    // pub fn new(method: Method, path: impl Into<Cow<'static, str>>) {}

    pub fn get(path: impl Into<Cow<'static, str>>) -> Self {
        Self {
            method: "GET",
            path: path.into(),
        }
    }

    pub fn post(path: impl Into<Cow<'static, str>>) -> Self {
        Self {
            method: "POST",
            path: path.into(),
        }
    }

    pub fn put(path: impl Into<Cow<'static, str>>) -> Self {
        Self {
            method: "PUT",
            path: path.into(),
        }
    }

    pub fn patch(path: impl Into<Cow<'static, str>>) -> Self {
        Self {
            method: "PATCH",
            path: path.into(),
        }
    }

    pub fn delete(path: impl Into<Cow<'static, str>>) -> Self {
        Self {
            method: "DELETE",
            path: path.into(),
        }
    }

    // TODO: Configure other OpenAPI stuff like auth???

    pub fn build<TError, TThisCtx, TThisInput, TThisResult>(
        self,
    ) -> Middleware<TError, TThisCtx, TThisInput, TThisResult>
    where
        TError: 'static,
        TThisCtx: Send + 'static,
        TThisInput: Send + 'static,
        TThisResult: Send + 'static,
    {
        // TODO: Can we have a middleware with only a `setup` function to avoid the extra future boxing???
        Middleware::new(|ctx, input, next| async move { next.exec(ctx, input).await }).setup(
            move |state, meta| {
                state
                    .get_mut_or_init::<OpenAPIState>(Default::default)
                    .0
                    .insert((self.method, self.path), meta.name().to_string());
            },
        )
    }
}

// The state that is stored into rspc.
// A map of (method, path) to procedure name.
#[derive(Default)]
struct OpenAPIState(HashMap<(&'static str, Cow<'static, str>), String>);

// TODO: Axum should be behind feature flag
// TODO: Can we decouple webserver from OpenAPI while keeping something maintainable????
pub fn mount<TCtx, S>(
    router: Router2<TCtx>,
    // TODO: Make Axum extractors work
    ctx_fn: impl Fn(&Parts) -> TCtx + Clone + Send + Sync + 'static,
) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + 'static,
{
    let mut r = axum::Router::new();

    // let mut paths: HashMap<_, HashMap<_, _>> = HashMap::new();
    // if let Some(endpoints) = router.state.get::<OpenAPIState>() {
    //     for ((method, path), procedure_name) in endpoints.0.iter() {
    //         let procedure = router
    //             .into_iter()
    //             .find(|(k, _)| k.join(".") == *procedure_name)
    //             // .get(&Cow::Owned(procedure_name.clone()))
    //             .expect("unreachable: a procedure was registered that doesn't exist")
    //             .clone();
    //         let ctx_fn = ctx_fn.clone();

    //         paths
    //             .entry(path.clone())
    //             .or_default()
    //             .insert(method.to_lowercase(), procedure.clone());

    //         r = r.route(
    //             path,
    //             match *method {
    //                 "GET" => {
    //                     // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
    //                     get(
    //                         move |parts: Parts, query: Query<HashMap<String, String>>| async move {
    //                             let ctx = (ctx_fn)(&parts);

    //                             handle_procedure(
    //                                 ctx,
    //                                 &mut serde_json::Deserializer::from_str(
    //                                     query.get("input").map(|v| &**v).unwrap_or("null"),
    //                                 ),
    //                                 procedure,
    //                             )
    //                             .await
    //                         },
    //                     )
    //                 }
    //                 "POST" => {
    //                     // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
    //                     post(move |parts: Parts, body: Bytes| async move {
    //                         let ctx = (ctx_fn)(&parts);

    //                         handle_procedure(
    //                             ctx,
    //                             &mut serde_json::Deserializer::from_slice(&body),
    //                             procedure,
    //                         )
    //                         .await
    //                     })
    //                 }
    //                 "PUT" => {
    //                     // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
    //                     put(move |parts: Parts, body: Bytes| async move {
    //                         let ctx = (ctx_fn)(&parts);

    //                         handle_procedure(
    //                             ctx,
    //                             &mut serde_json::Deserializer::from_slice(&body),
    //                             procedure,
    //                         )
    //                         .await
    //                     })
    //                 }
    //                 "PATCH" => {
    //                     // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
    //                     patch(move |parts: Parts, body: Bytes| async move {
    //                         let ctx = (ctx_fn)(&parts);

    //                         handle_procedure(
    //                             ctx,
    //                             &mut serde_json::Deserializer::from_slice(&body),
    //                             procedure,
    //                         )
    //                         .await
    //                     })
    //                 }
    //                 "DELETE" => {
    //                     // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
    //                     delete(move |parts: Parts, body: Bytes| async move {
    //                         let ctx = (ctx_fn)(&parts);

    //                         handle_procedure(
    //                             ctx,
    //                             &mut serde_json::Deserializer::from_slice(&body),
    //                             procedure,
    //                         )
    //                         .await
    //                     })
    //                 }
    //                 _ => panic!("Unsupported method"),
    //             },
    //         );
    //     }
    // }

    // let schema = Arc::new(json!({
    //   "openapi": "3.0.3",
    //   "info": {
    //     "title": "rspc OpenAPI",
    //     "description": "This is a demo of rspc OpenAPI",
    //     "version": "0.0.0"
    //   },
    //   "paths": paths.into_iter()
    //     .map(|(path, procedures)| {
    //         let mut methods = HashMap::new();
    //         for (method, procedure) in procedures {
    //             methods.insert(method.to_string(), json!({
    //                 "operationId": procedure.ty().key.to_string(),
    //                 "responses": {
    //                     "200": {
    //                         "description": "Successful operation"
    //                     }
    //                 }
    //             }));
    //         }

    //         (path, methods)
    //     })
    //     .collect::<HashMap<_, _>>()
    // })); // TODO: Maybe convert to string now cause it will be more efficient to clone

    r.route(
        // TODO: Allow the user to configure this URL & turn it off
        "/api/docs",
        get(|| async { Html(include_str!("swagger.html")) }),
    )
    // .route(
    //     // TODO: Allow the user to configure this URL & turn it off
    //     "/api/openapi.json",
    //     get(move || async move { Json((*schema).clone()) }),
    // )
}

// Used for `GET` and `POST` endpoints
// async fn handle_procedure<'a, 'de, TCtx>(
//     ctx: TCtx,
//     input: DynInput<'a, 'de>,
//     procedure: Procedure2<TCtx>,
// ) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
//     let mut stream = procedure.exec(ctx, input).map_err(|err| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({
//                 // TODO: This or not?
//                 "_rspc_error": err.to_string()
//             })),
//         )
//     })?;

//     // TODO: Support for streaming
//     while let Some(value) = stream.next().await {
//         // TODO: We should probs deserialize into buffer instead of value???
//         return match value.map(|v| v.serialize(serde_json::value::Serializer)) {
//             Ok(Ok(value)) => Ok(Json(value)),
//             Ok(Err(err)) => Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({
//                     "_rspc_error": err.to_string()
//                 })),
//             )),
//             Err(err) => Err((
//                 StatusCode::from_u16(err.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
//                 Json(
//                     err.serialize(serde_json::value::Serializer)
//                         .map_err(|err| {
//                             (
//                                 StatusCode::INTERNAL_SERVER_ERROR,
//                                 Json(json!({
//                                     "_rspc_error": err.to_string()
//                                 })),
//                             )
//                         })?,
//                 ),
//             )),
//         };
//     }

//     Ok(Json(serde_json::Value::Null))
// }
