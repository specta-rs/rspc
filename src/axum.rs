use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    routing::{MethodFilter, MethodRouter},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{KeyDefinition, Router};

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

impl<TCtx: Send + Sync, TQueryKey: KeyDefinition, TMutationKey: KeyDefinition>
    Router<TCtx, TQueryKey, TMutationKey>
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
    ) -> Json<Vec<Response>> {
        let result = self
            .exec_query_unsafe(ctx_fn(), key, serde_json::from_str(&params.input).unwrap())
            .await
            .unwrap();

        Json(vec![Response::Success {
            id: None,
            result: Result::Data(result),
        }])
    }

    async fn post(
        self: Arc<Self>,
        ctx_fn: fn() -> TCtx,
        Path(key): Path<String>,
        Query(_params): Query<PostParams>,
        Json(input): Json<Value>,
    ) -> Json<Vec<Response>> {
        let result = self
            .exec_mutation_unsafe(ctx_fn(), key, input)
            .await
            .unwrap();

        Json(vec![Response::Success {
            id: None,
            result: Result::Data(result),
        }])
    }
}
