use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{MethodFilter, MethodRouter},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CompiledRouter, KeyDefinition};

#[derive(Debug, Deserialize)]
pub struct GetParams {
    pub batch: i32, // TODO: is this correct number type?
    pub input: String,
}

#[derive(Debug, Deserialize)]
pub struct PostParams {
    pub batch: i32, // TODO: is this correct number type?
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum Result {
    Data(Value),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Response {
    Error(()),
    Success { id: Option<String>, result: Result },
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
    CompiledRouter<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    pub fn axum_handler(self: Arc<Self>, ctx_fn: fn() -> TCtx) -> MethodRouter {
        let get_this = self.clone();
        let post_this = self;
        MethodRouter::new()
            .on(MethodFilter::GET, move |path, query| {
                get_this.get(ctx_fn, path, query)
            })
            .on(MethodFilter::POST, move |path, query, body| {
                post_this.post(ctx_fn, path, query, body)
            })
    }

    async fn get(
        self: Arc<Self>,
        ctx_fn: fn() -> TCtx,
        Path(key): Path<String>,
        Query(params): Query<GetParams>,
    ) -> impl IntoResponse {
        match serde_json::from_str(&params.input) {
            Ok(mut arg) => {
                if let Value::Object(obj) = &arg {
                    if obj.len() == 0 {
                        arg = Value::Null;
                    }
                }

                if let Value::Object(obj) = &arg {
                    if obj.len() == 1 {
                        if let Some(v) = obj.get("0") {
                            arg = v.clone();
                        }
                    }
                }

                match self.exec_query_unsafe(ctx_fn(), key, arg).await {
                    Ok(result) => (
                        StatusCode::OK,
                        Json(vec![Response::Success {
                            id: None,
                            result: Result::Data(result),
                        }]),
                    ),

                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(vec![Response::Error(())]),
                    ),
                }
            }
            Err(_) => (StatusCode::BAD_REQUEST, Json(vec![Response::Error(())])),
        }
    }

    async fn post(
        self: Arc<Self>,
        ctx_fn: fn() -> TCtx,
        Path(key): Path<String>,
        Query(_params): Query<PostParams>,
        Json(mut arg): Json<Value>,
    ) -> impl IntoResponse {
        if let Value::Object(obj) = &arg {
            if obj.len() == 0 {
                arg = Value::Null;
            }
        }

        if let Value::Object(obj) = &arg {
            if obj.len() == 1 {
                if let Some(v) = obj.get("0") {
                    arg = v.clone();
                }
            }
        }

        match self.exec_mutation_unsafe(ctx_fn(), key, arg).await {
            Ok(result) => (
                StatusCode::OK,
                Json(vec![Response::Success {
                    id: None,
                    result: Result::Data(result),
                }]),
            ),

            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![Response::Error(())]),
            ),
        }
    }
}
