use std::{collections::HashMap, future::poll_fn, sync::Arc, task::Poll};

use axum::{
    body::Bytes,
    extract::Query,
    http::{header, request::Parts, HeaderMap, StatusCode},
    routing::{get, post},
    Json,
};
use futures::StreamExt;
use rspc::{
    procedure::{Procedure, ProcedureInput, ProcedureKind},
    BuiltRouter,
};
use serde_json::json;

/// Construct a new [`axum::Router`](axum::Router) to expose a given [`rspc::Router`](rspc::Router).
pub struct Endpoint<TCtx> {
    router: BuiltRouter<TCtx>,
    endpoints: bool,
    websocket: Option<fn(&TCtx) -> TCtx>,
    batching: bool,
}

impl<TCtx: Send + Sync + 'static> Endpoint<TCtx> {
    /// Construct a new [`axum::Router`](axum::Router) with all features enabled.
    ///
    /// This will enable all features, if you want to configure which features are enabled you can use [`Endpoint::builder`] instead.
    ///
    /// # Usage
    ///
    /// ```rust
    /// axum::Router::new().nest(
    ///     "/rspc",
    ///     rspc_axum::Endpoint::new(rspc::Router::new().build().unwrap(), || ()),
    /// );
    /// ```
    pub fn new<S>(
        router: BuiltRouter<TCtx>,
        // TODO: Parse this to `Self::build` -> It will make rustfmt result way nicer
        // TODO: Make Axum extractors work
        ctx_fn: impl Fn(&Parts) -> TCtx + Send + Sync + 'static,
    ) -> axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
        // TODO: Error type???
        // F: Future<Output = Result<TCtx, ()>> + Send + Sync + 'static,
        TCtx: Clone,
    {
        let mut t = Self::builder(router).with_endpoints();
        #[cfg(feature = "ws")]
        {
            t = t.with_websocket();
        }
        t.with_batching().build(ctx_fn)
    }

    /// Construct a new [`Endpoint`](Endpoint) with no features enabled.
    ///
    /// # Usage
    ///
    /// ```rust
    /// axum::Router::new().nest(
    ///     "/rspc",
    ///     rspc_axum::Endpoint::builder(rspc::Router::new().build().unwrap())
    ///         // Exposes HTTP endpoints for queries and mutations.
    ///         .with_endpoints()
    ///         // Exposes a Websocket connection for queries, mutations and subscriptions.
    ///         .with_websocket()
    ///         // Enables support for the frontend sending batched queries.
    ///         .with_batching()
    ///         .build(|| ()),
    /// );   
    /// ```
    pub fn builder(router: BuiltRouter<TCtx>) -> Self {
        Self {
            router,
            endpoints: false,
            websocket: None,
            batching: false,
        }
    }

    /// Enables HTTP endpoints for queries and mutations.
    ///
    /// This is exposed as `/routerName.procedureName`
    pub fn with_endpoints(mut self) -> Self {
        Self {
            endpoints: true,
            ..self
        }
    }

    /// Exposes a Websocket connection for queries, mutations and subscriptions.
    ///
    /// This is exposed as a `/ws` endpoint.
    #[cfg(feature = "ws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    pub fn with_websocket(self) -> Self
    where
        TCtx: Clone,
    {
        Self {
            websocket: Some(|ctx| ctx.clone()),
            ..self
        }
    }

    /// Enables support for the frontend sending batched queries.
    ///
    /// This is exposed as a `/_batch` endpoint.
    pub fn with_batching(self) -> Self
    where
        TCtx: Clone,
    {
        Self {
            batching: true,
            ..self
        }
    }

    // TODO: Make Axum extractors work
    // TODO: Async or `Result` return type for context function
    /// Build an [`axum::Router`](axum::Router) with the configured features.
    pub fn build<S>(
        self,
        ctx_fn: impl Fn(&Parts) -> TCtx + Send + Sync + 'static,
    ) -> axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let mut r = axum::Router::new();
        let ctx_fn = Arc::new(ctx_fn);

        if self.endpoints {
            for (key, procedure) in &self.router.procedures {
                let ctx_fn = ctx_fn.clone();
                let procedure = procedure.clone();
                r = match procedure.kind() {
                    ProcedureKind::Query => {
                        r.route(
                            &format!("/{}", key),
                            // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
                            get(
                                move |parts: Parts,
                                      query: Query<HashMap<String, String>>,
                                        | async move {
                                    let ctx = (ctx_fn)(&parts);

                                    handle_procedure(
                                        ctx,
                                        &mut serde_json::Deserializer::from_str(
                                            query.get("input").map(|v| &**v).unwrap_or("null"),
                                        ),
                                        parts.headers,
                                        procedure,
                                    )
                                    .await
                                },
                            ),
                        )
                    }
                    ProcedureKind::Mutation => r.route(
                        &format!("/{}", key),
                        // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
                        post(move |parts: Parts, body: Bytes| async move {
                            let ctx = (ctx_fn)(&parts);

                            handle_procedure(
                                ctx,
                                &mut serde_json::Deserializer::from_slice(&body),
                                parts.headers,
                                procedure,
                            )
                            .await
                        }),
                    ),
                    ProcedureKind::Subscription => continue,
                };
            }
        }

        #[cfg(feature = "ws")]
        if let Some(clone_ctx) = self.websocket {
            use axum::extract::ws::WebSocketUpgrade;
            r = r.route(
                "/ws",
                get(move |parts: Parts, ws: WebSocketUpgrade| async move {
                    let ctx = (ctx_fn)(&parts);

                    ws.on_upgrade(move |socket| async move {
                        todo!();

                        // while let Some(msg) = socket.recv().await {}
                    })
                }),
            );
        }

        if self.batching {
            // TODO: Support for batching & stream batching

            // todo!();
        }

        r.with_state(())
    }
}

// Used for `GET` and `POST` endpoints
// TODO: We should probs deserialize into buffer instead of value all over this function!!!!
async fn handle_procedure<'de, TCtx>(
    ctx: TCtx,
    input: impl ProcedureInput<'de>,
    headers: HeaderMap,
    procedure: Procedure<TCtx>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let is_legacy_client = headers.get("x-rspc").is_none();

    let mut stream = procedure.exec(ctx, input).map_err(|err| {
        if is_legacy_client {
            (
                StatusCode::OK,
                Json(json!({
                    "jsonrpc":"2.0",
                    "id":null,
                    "result":{
                        "type":"error",
                        "data": {
                            "code": 500,
                            "message": err.to_string(),
                            "data": null
                        }
                    }
                })),
            )
        } else {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "_rspc_error": err.to_string()
                })),
            )
        }
    })?;

    if is_legacy_client {
        let value = match stream.next().await {
            Some(value) => {
                if stream.next().await.is_some() {
                    println!("Streaming was attempted with a legacy rspc client! Ensure your not using `rspc::Stream` unless your clients are up to date.");
                }

                value
            }
            None => {
                return Ok(Json(json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "result": {
                        "type": "response",
                        "data": "todo"
                    }
                })));
            }
        };

        return Ok(Json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "result": match value
                .map_err(|err| (err.status(), err.to_string(), err.serialize(serde_json::value::Serializer).unwrap_or_default()))
                .and_then(|v| v.serialize(serde_json::value::Serializer).map_err(|err| (500, err.to_string(), serde_json::Value::Null))) {
                Ok(value) => {
                    json!({
                        "type": "response",
                        "data": value,
                    })
                }
                Err((status, message, data)) => {
                    json!({
                        "type": "error",
                        "data": {
                            "code": status,
                            "message": message,
                            // `data` was technically always `null` in legacy rspc but we'll include it how it was intended.
                            "data": data,
                        }
                    })
                }
            }
        })));
    } else {
        // TODO: Support for streaming
        while let Some(value) = stream.next().await {
            return match value.map(|v| v.serialize(serde_json::value::Serializer)) {
                Ok(Ok(value)) => Ok(Json(value)),
                Ok(Err(err)) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "_rspc_error": err.to_string()
                    })),
                )),
                Err(err) => Err((
                    StatusCode::from_u16(err.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    Json(
                        err.serialize(serde_json::value::Serializer)
                            .map_err(|err| {
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({
                                        "_rspc_error": err.to_string()
                                    })),
                                )
                            })?,
                    ),
                )),
            };
        }

        Ok(Json(serde_json::Value::Null))
    }
}
