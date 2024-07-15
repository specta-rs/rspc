use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use futures::StreamExt;
use rspc::{procedure::ProcedureKind, BuiltRouter};

pub struct Endpoint<TCtx> {
    router: BuiltRouter<TCtx>,
    axum: axum::Router<()>,
    ctx_fn: Arc<dyn Fn() -> TCtx + Send + Sync>,
}

impl<TCtx: Send + Sync + 'static> Endpoint<TCtx> {
    // TODO: Async or `Result` return type for context function
    pub fn new(
        router: BuiltRouter<TCtx>,
        // TODO: Parse this to `Self::build` -> It will make rustfmt result way nicer
        ctx_fn: impl Fn() -> TCtx + Send + Sync + 'static,
    ) -> Self {
        Self {
            router,
            axum: axum::Router::new(),
            ctx_fn: Arc::new(ctx_fn),
        }
    }

    // TODO: What to call this???
    pub fn with_endpoints(mut self) -> Self {
        for (key, procedure) in &self.router.procedures {
            let ctx_fn = self.ctx_fn.clone();
            let procedure = procedure.clone();
            self.axum = match procedure.kind() {
                ProcedureKind::Query => self.axum.route(
                    &format!("/{}", key),
                    // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
                    get(move |query: Query<HashMap<String, String>>| async move {
                        let ctx = (ctx_fn)();

                        let mut stream = procedure
                            .exec(
                                ctx,
                                &mut serde_json::Deserializer::from_str(
                                    query.get("input").map(|v| &**v).unwrap_or("null"),
                                ),
                            )
                            .map_err(|err| {
                                // TODO: Error code by matching off `InternalError`
                                (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))
                                    .into_response()
                            })?;

                        // TODO: Support for streaming
                        while let Some(value) = stream.next().await {
                            return match value.map(|v| v.serialize(serde_json::value::Serializer)) {
                                Ok(Ok(value)) => Ok(Json(value)),
                                Ok(Err(err)) => {
                                    // TODO: Error code by matching off `InternalError`
                                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))
                                        .into_response())
                                }
                                Err(err) => panic!("{err:?}"), // TODO: Error handling -> How to serialize `TError`??? -> Should this be done in procedure?
                            };
                        }

                        Ok::<_, Response>(Json(serde_json::Value::Null))
                    }),
                ),
                ProcedureKind::Mutation => self.axum.route(
                    &format!("/{}", key),
                    // TODO: By moving `procedure` into the closure we hang onto the types for the duration of the program which is probs undesirable.
                    post(move |body: Bytes| async move {
                        let ctx = (ctx_fn)();

                        let mut stream = procedure
                            .exec(ctx, &mut serde_json::Deserializer::from_slice(&body))
                            .map_err(|err| {
                                // TODO: Error code by matching off `InternalError`
                                (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))
                                    .into_response()
                            })?;

                        // TODO: Support for streaming
                        while let Some(value) = stream.next().await {
                            return match value.map(|v| v.serialize(serde_json::value::Serializer)) {
                                Ok(Ok(value)) => Ok(Json(value)),
                                Ok(Err(err)) => {
                                    // TODO: Error code by matching off `InternalError`
                                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))
                                        .into_response())
                                }
                                Err(err) => panic!("{err:?}"), // TODO: Error handling -> How to serialize `TError`??? -> Should this be done in procedure?
                            };
                        }

                        Ok::<_, Response>(Json(serde_json::Value::Null))
                    }),
                ),
                ProcedureKind::Subscription => continue,
            };
        }

        self
    }

    // TODO: Put behind feature flag
    pub fn with_websocket(self) -> Self
    where
        TCtx: Clone,
    {
        Self {
            axum: self.axum.route(
                "/ws",
                get(|| async move {
                    // TODO: Support for websockets
                    "this is rspc websocket"
                }),
            ),
            ..self
        }
    }

    pub fn with_batching(self) -> Self
    where
        TCtx: Clone,
    {
        // TODO: Support for batching & stream batching

        self
    }

    pub fn build<S: Clone + Send + Sync + 'static>(self) -> axum::Router<S> {
        self.axum.with_state(())
    }
}
