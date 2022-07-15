use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query,
    },
    http::StatusCode,
    response::IntoResponse,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageMethod {
    Query,
    Mutation,
    SubscriptionAdd,
    SubscriptionRemove,
}

#[derive(Debug, Deserialize)]
pub struct WsRequest {
    pub id: String,
    pub method: MessageMethod,
    pub operation: String,
    pub arg: Value,
}

#[derive(Debug, Serialize)]
pub struct WsResponse {
    pub id: String,
    pub result: WsResponseBody,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum WsResponseBody {
    Error(()),
    Success(Value),
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
    Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
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

    pub fn axum_ws_handler(self: Arc<Self>, ctx_fn: fn() -> TCtx) -> MethodRouter {
        MethodRouter::new().on(MethodFilter::GET, move |ws: WebSocketUpgrade| async move {
            ws.on_upgrade(move |socket| async move { self.ws(ctx_fn, socket).await })
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

    async fn ws(self: Arc<Self>, ctx_fn: fn() -> TCtx, mut socket: WebSocket) {
        while let Some(msg) = socket.recv().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(msg) => {
                        let result = match serde_json::from_str::<WsRequest>(&msg) {
                            Ok(mut msg) => {
                                if let Value::Object(obj) = &msg.arg {
                                    if obj.len() == 0 {
                                        msg.arg = Value::Null;
                                    }
                                }

                                if let Value::Object(obj) = &msg.arg {
                                    if obj.len() == 1 {
                                        if let Some(v) = obj.get("0") {
                                            msg.arg = v.clone();
                                        }
                                    }
                                }

                                let result = match msg.method {
                                    MessageMethod::Query => {
                                        self.exec_query_unsafe(ctx_fn(), msg.operation, msg.arg)
                                            .await
                                    }
                                    MessageMethod::Mutation => {
                                        self.exec_mutation_unsafe(ctx_fn(), msg.operation, msg.arg)
                                            .await
                                    }
                                    MessageMethod::SubscriptionAdd => {
                                        self.exec_subscription_unsafe(msg.operation)
                                            .await
                                            .map(|_| Value::Null) // TODO: This doesn't need a response
                                    }
                                    MessageMethod::SubscriptionRemove => {
                                        unimplemented!(); // TODO: Make this work
                                    }
                                };

                                WsResponse {
                                    id: msg.id,
                                    result: match result {
                                        Ok(result) => WsResponseBody::Success(result),
                                        Err(_) => WsResponseBody::Error(()),
                                    },
                                }
                            }
                            Err(_) => WsResponse {
                                id: "_".into(), // TODO: Is this a good idea? What does TRPC do in this case?
                                result: WsResponseBody::Error(()),
                            },
                        };

                        if socket
                            .send(Message::Text(serde_json::to_string(&result).unwrap()))
                            .await
                            .is_err()
                        {
                            // client disconnected
                            return;
                        }
                    }
                    Message::Binary(_) => {
                        // TODO
                        println!("CLIENT SENT UNSUPPORTED WEBSOCKET OPERATION 'Binary'!");
                    }
                    Message::Ping(_) => {
                        // TODO
                        println!("CLIENT SENT UNSUPPORTED WEBSOCKET OPERATION 'Ping'!");
                    }
                    Message::Pong(_) => {
                        // TODO
                        println!("CLIENT SENT UNSUPPORTED WEBSOCKET OPERATION 'Pong'!");
                    }
                    Message::Close(_) => {}
                }
            } else {
                // client disconnected
                return;
            };
        }
    }
}
