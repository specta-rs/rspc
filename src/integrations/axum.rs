use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequest, Path, Query, RequestParts,
    },
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{MethodFilter, MethodRouter},
    Json,
};

use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
    utils::{self, MessageMethod, Response, ResponseResult},
    KeyDefinition, Router,
};

#[derive(Debug, Deserialize)]
pub struct GetParams {
    pub batch: Option<i32>, // TODO: is this correct number type?
    pub input: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostParams {
    pub batch: i32, // TODO: is this correct number type?
}

pub enum TCtxFuncResult<'a, TCtx> {
    Value(TCtx),
    Future(Pin<Box<dyn Future<Output = Result<TCtx, axum::response::Response>> + Send + 'a>>),
}

// TODO: This request extractor system needs a huge refactor!!!!

pub trait TCtxFunc<TCtx, TMarker>: Clone + Send + Sync + 'static
where
    TCtx: Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Body>) -> TCtxFuncResult<'a, TCtx>;
}

pub struct NoArgMarker(PhantomData<()>);
impl<TCtx, TFunc> TCtxFunc<TCtx, NoArgMarker> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce() -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'a>(&self, _request: &'a mut RequestParts<Body>) -> TCtxFuncResult<'a, TCtx> {
        TCtxFuncResult::Value(self.clone()())
    }
}

pub struct OneArgAxumRequestMarker<T1>(PhantomData<T1>);
impl<T1: FromRequest<Body> + Send + 'static, TCtx, TFunc>
    TCtxFunc<TCtx, OneArgAxumRequestMarker<T1>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1) -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Body>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            match T1::from_request(request).await {
                Ok(t1) => Ok(this(t1)),
                Err(e) => Err(e.into_response()),
            }
        }))
    }
}

// TODO: Build macro so we can support up to 16 different extractor arguments like Axum

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
    Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    pub fn axum_handler<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
    ) -> MethodRouter {
        let get_this = self.clone();
        let post_this = self;
        let ctx_fn2 = ctx_fn.clone();
        MethodRouter::new()
            .on(MethodFilter::GET, move |path, query, request| {
                get_this.get(ctx_fn, path, query, request)
            })
            .on(MethodFilter::POST, move |path, query, body, request| {
                post_this.post(ctx_fn2, path, query, body, request)
            })
    }

    pub fn axum_ws_handler<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
    ) -> MethodRouter {
        MethodRouter::new().on(
            MethodFilter::GET,
            move |ws: WebSocketUpgrade, request| async move {
                ws.on_upgrade(move |socket| async move { self.ws(ctx_fn, socket, request).await })
            },
        )
    }

    async fn get<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
        Path(key): Path<String>,
        Query(params): Query<GetParams>,
        request: Request<Body>,
    ) -> impl IntoResponse {
        let mut request_parts = RequestParts::new(request);
        match params
            .input
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(Value::Null))
        {
            Ok(arg) => {
                let ctx = match ctx_fn.exec(&mut request_parts) {
                    TCtxFuncResult::Value(ctx) => ctx,
                    TCtxFuncResult::Future(future) => match future.await {
                        Ok(ctx) => ctx,
                        Err(_) => {
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(vec![Response::Response(ResponseResult::Error)]),
                            );
                        }
                    },
                };

                (
                    StatusCode::OK, // TODO: Make status code correct based on `Response`
                    Json(vec![
                        utils::Request {
                            id: None,
                            method: MessageMethod::Query,
                            operation: key,
                            arg: Some(arg),
                        }
                        .handle(ctx, &self, None)
                        .await,
                    ]),
                )
            }
            Err(_) => (
                StatusCode::BAD_REQUEST,
                Json(vec![Response::Response(ResponseResult::Error)]),
            ),
        }
    }

    async fn post<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
        Path(key): Path<String>,
        Query(params): Query<PostParams>,
        Json(arg): Json<Option<Value>>,
        request: Request<Body>,
    ) -> impl IntoResponse {
        let mut request_parts = RequestParts::new(request);

        let ctx = match ctx_fn.exec(&mut request_parts) {
            TCtxFuncResult::Value(ctx) => ctx,
            TCtxFuncResult::Future(future) => match future.await {
                Ok(ctx) => ctx,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(vec![Response::Response(ResponseResult::Error)]),
                    );
                }
            },
        };

        (
            StatusCode::OK, // TODO: Make status code correct based on `Response`
            Json(vec![
                utils::Request {
                    id: None,
                    method: MessageMethod::Mutation,
                    operation: key,
                    arg,
                }
                .handle(ctx, &self, None)
                .await,
            ]),
        )
    }

    async fn ws<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
        mut socket: WebSocket,
        request: Request<Body>,
    ) {
        let mut request_parts = RequestParts::new(request);
        let (tx, mut rx) = mpsc::unbounded_channel::<utils::Response>();
        loop {
            tokio::select! {
            msg = socket.recv() => {
                    match msg {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(msg) => {
                                    let result = match serde_json::from_str::<utils::Request>(&msg) {
                                        Ok(result) => {
                                            let ctx = match ctx_fn.exec(&mut request_parts) {
                                                TCtxFuncResult::Value(ctx) => ctx,
                                                TCtxFuncResult::Future(future) => match future.await {
                                                    Ok(ctx) => ctx,
                                                    Err(err) => {
                                                        println!("ERROR GETTING CONTEXT! {:?}", err); // TODO: Error handling here
                                                        return;
                                                    }
                                                },
                                            };

                                            result.handle(ctx, &self, Some(&tx)).await
                                        },
                                        Err(_) => utils::Response::Response (ResponseResult::Error),
                                    };

                                    if !matches!(result, utils::Response::None) && socket
                                        .send(Message::Text(serde_json::to_string(&result).unwrap())) // TODO: Error handling
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
                        }
                        _ => {
                            break;
                        }
                    }
                }
                msg = rx.recv() => {
                    match socket
                        .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                        .await
                    {
                        Ok(_) => {},
                        Err(_) => {
                            println!("ERROR SENDING MESSAGE!"); // TODO: Error handling here
                            return;
                        }
                    }
                }
            }
        }
    }
}
