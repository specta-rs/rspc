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

use crate::{ClientContext, Error, ErrorCode, ExecError, OperationKey, OperationKind, Router};

#[derive(Debug, Deserialize)]
pub struct GetParams {
    pub batch: Option<i32>,
    pub input: Option<String>,
    pub margs: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostParams {
    pub batch: Option<i32>,
}

pub enum TCtxFuncResult<'a, TCtx> {
    Value(Result<TCtx, ExecError>),
    Future(Pin<Box<dyn Future<Output = Result<TCtx, ExecError>> + Send + 'a>>),
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
        TCtxFuncResult::Value(Ok(self.clone()()))
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
                Err(_) => Err(ExecError::AxumExtractorError),
            }
        }))
    }
}

// TODO: Build macro so we can support up to 16 different extractor arguments like Axum

impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
{
    pub fn axum_handler<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
    ) -> MethodRouter {
        let get_this = self.clone();
        let post_this = self;
        let client_ctx = ClientContext::new();

        MethodRouter::new()
            .on(MethodFilter::GET, {
                let ctx_fn = ctx_fn.clone();
                let client_ctx = client_ctx.clone();
                move |path, query, request| get_this.get(ctx_fn, client_ctx, path, query, request)
            })
            .on(MethodFilter::POST, move |path, query, request| {
                post_this.post(ctx_fn, client_ctx, path, query, request) // body
            })
    }

    pub fn axum_ws_handler<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
    ) -> MethodRouter {
        let client_ctx = ClientContext::new();

        MethodRouter::new().on(
            MethodFilter::GET,
            move |ws: WebSocketUpgrade, request| async move {
                ws.on_upgrade(move |socket| async move {
                    self.ws(ctx_fn, client_ctx, socket, request).await
                })
            },
        )
    }

    async fn get<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
        client_ctx: Arc<ClientContext>,
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
                let resp = match ctx_fn.exec(&mut request_parts) {
                    TCtxFuncResult::Value(ctx) => match ctx {
                        Ok(ctx) => {
                            crate::Request {
                                id: None,
                                operation: OperationKind::Query,
                                key: OperationKey(key, Some(arg)),
                            }
                            .handle(ctx, &self, &client_ctx, None)
                            .await
                        }
                        Err(err) => err.into_rspc_err().into_response(None),
                    },
                    TCtxFuncResult::Future(future) => match future.await {
                        Ok(ctx) => {
                            crate::Request {
                                id: None,
                                operation: OperationKind::Query,
                                key: OperationKey(key, Some(arg)),
                            }
                            .handle(ctx, &self, &client_ctx, None)
                            .await
                        }
                        Err(err) => err.into_rspc_err().into_response(None),
                    },
                };

                (
                    StatusCode::OK, // TODO: Make status code correct based on `Response`
                    Json(vec![resp]),
                )
            }
            Err(_) => (StatusCode::BAD_REQUEST, Json(vec![])),
        }
    }

    async fn post<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
        client_ctx: Arc<ClientContext>,
        Path(key): Path<String>,
        Query(_params): Query<PostParams>,
        request: Request<Body>,
    ) -> impl IntoResponse {
        let mut request_parts = RequestParts::new(request);
        let arg = match Json::<Option<Value>>::from_request(&mut request_parts).await {
            Ok(t1) => t1.0,
            Err(e) => return (StatusCode::BAD_REQUEST, e.into_response()).into_response(),
        };

        let resp = match ctx_fn.exec(&mut request_parts) {
            TCtxFuncResult::Value(ctx) => match ctx {
                Ok(ctx) => {
                    crate::Request {
                        id: None,
                        operation: OperationKind::Mutation,
                        key: OperationKey(key, arg),
                    }
                    .handle(ctx, &self, &client_ctx, None)
                    .await
                }
                Err(err) => err.into_rspc_err().into_response(None),
            },
            TCtxFuncResult::Future(future) => match future.await {
                Ok(ctx) => {
                    crate::Request {
                        id: None,
                        operation: OperationKind::Mutation,
                        key: OperationKey(key, arg),
                    }
                    .handle(ctx, &self, &client_ctx, None)
                    .await
                }
                Err(err) => err.into_rspc_err().into_response(None),
            },
        };

        (
            StatusCode::OK, // TODO: Make status code correct based on `Response`
            Json(vec![resp]),
        )
            .into_response()
    }

    async fn ws<TMarker>(
        self: Arc<Self>,
        ctx_fn: impl TCtxFunc<TCtx, TMarker>,
        client_ctx: Arc<ClientContext>,
        mut socket: WebSocket,
        request: Request<Body>,
    ) {
        let mut request_parts = RequestParts::new(request);
        let (tx, mut rx) = mpsc::unbounded_channel::<crate::Response>();
        loop {
            tokio::select! {
            msg = socket.recv() => {
                    match msg {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(msg) => {
                                    let result = match serde_json::from_str::<crate::Request>(&msg) {
                                        Ok(result) => {
                                            match ctx_fn.exec(&mut request_parts) {
                                                TCtxFuncResult::Value(ctx) => match ctx {
                                                    Ok(ctx) => result.handle(ctx, &self, &client_ctx, Some(&tx)).await,
                                                    Err(err) => err.into_rspc_err().into_response(result.id),
                                                },
                                                TCtxFuncResult::Future(future) => match future.await {
                                                    Ok(ctx) => result.handle(ctx, &self, &client_ctx, Some(&tx)).await,
                                                    Err(err) => {
                                                        Error {
                                                            code: ErrorCode::InternalServerError,
                                                            message: err.to_string(),
                                                        }.into_response(result.id)
                                                    }
                                                },
                                            }
                                        },
                                        Err(err) => ExecError::DeserializingArgErr(err).into_rspc_err().into_response(None),
                                    };

                                    if !matches!(result, crate::Response::None) && socket
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
