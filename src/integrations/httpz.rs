pub use super::httpz_extractors::*;
pub use super::httpz_extractors::*;
use futures::{SinkExt, StreamExt};
use httpz::{
    cookie::CookieJar,
    http::{Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    Endpoint, GenericEndpoint, HttpEndpoint, HttpResponse, Request,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

use crate::{
    internal::{
        jsonrpc::{self, handle_json_rpc, RequestId, Sender, SubscriptionMap},
        ProcedureKind,
    },
    Router,
};

impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub fn endpoint<TCtxFnMarker: Send + Sync + 'static, TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>>(
        self: Arc<Self>,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        self.internal_endpoint(None, ctx_fn)
    }

    pub fn endpoint_with_prefix<
        TCtxFnMarker: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    >(
        self: Arc<Self>,
        url_prefix: &'static str,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        self.internal_endpoint(Some(url_prefix), ctx_fn)
    }

    fn internal_endpoint<
        TCtxFnMarker: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    >(
        self: Arc<Self>,
        url_prefix: Option<&'static str>,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        GenericEndpoint::new([Method::GET, Method::POST], move |req: Request| {
            // TODO: It would be nice if these clones weren't per request. Maybe httpz could allow context to be generated per thread and stored in thread local?
            let router = self.clone();
            let ctx_fn = ctx_fn.clone();

            async move {
                let websocket_url = format!("{}/ws", url_prefix.unwrap_or("/rspc")); // TODO: Match on variable in URL and not not the entire URL??
                let cookies = req.cookies();

                match (req.method(), req.uri().path()) {
                    (&Method::GET, url) if url == websocket_url => {
                        handle_websocket(ctx_fn, req, cookies, router).into_response()
                    }
                    (&Method::GET, _) => handle_http(
                        ctx_fn,
                        &format!("{}/", url_prefix.unwrap_or("/rspc")),
                        ProcedureKind::Query,
                        req,
                        cookies,
                        &router,
                    )
                    .await
                    .into_response(),
                    (&Method::POST, _) => handle_http(
                        ctx_fn,
                        &format!("{}/", url_prefix.unwrap_or("/rspc")),
                        ProcedureKind::Mutation,
                        req,
                        cookies,
                        &router,
                    )
                    .await
                    .into_response(),
                    _ => unreachable!(),
                }
            }
        })
    }
}

pub async fn handle_http<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    url_prefix: &str,
    kind: ProcedureKind,
    req: Request,
    cookies: CookieJar,
    router: &Arc<Router<TCtx>>,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let uri = req.uri().clone();

    let key = match uri.path().strip_prefix(url_prefix) {
        Some(key) => key,
        None => {
            return Ok((
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            )); // TODO: Include error information in response
        }
    };

    let input = match *req.method() {
        Method::GET => req
            .query_pairs()
            .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(None as Option<Value>)),
        Method::POST => (!req.body().is_empty())
            .then(|| serde_json::from_slice(req.body()))
            .unwrap_or(Ok(None)),
        _ => unreachable!(),
    };

    let input = match input {
        Ok(input) => input,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!(
                "Error passing parameters to operation '{}' with key '{:?}': {}",
                kind.to_str(),
                key,
                _err
            );

            return Ok((
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            ));
        }
    };

    #[cfg(feature = "tracing")]
    tracing::debug!(
        "Executing operation '{}' with key '{}' with params {:?}",
        kind.to_str(),
        key,
        input
    );

    let mut resp = Sender::Response(None);

    #[cfg(not(feature = "workers"))]
    let ctx = match ctx_fn.exec(&mut httpz::axum::axum::extract::RequestParts::new(
        req.into(),
    )) {
        TCtxFuncResult::Value(v) => v,
        TCtxFuncResult::Future(v) => v.await,
    };
    #[cfg(feature = "workers")]
    let ctx = match ctx_fn.exec() {
        TCtxFuncResult::Value(v) => v,
        TCtxFuncResult::Future(v) => v.await,
    };

    let ctx = match ctx {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Ok((
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            ));
        }
    };

    handle_json_rpc(
        ctx,
        jsonrpc::Request {
            jsonrpc: None,
            id: RequestId::Null,
            inner: match kind {
                ProcedureKind::Query => jsonrpc::RequestInner::Query {
                    path: key.to_string(),
                    input,
                },
                ProcedureKind::Mutation => jsonrpc::RequestInner::Mutation {
                    path: key.to_string(),
                    input,
                },
                ProcedureKind::Subscription => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Attempted to execute a subscription operation with HTTP");

                    return Ok((
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("Content-Type", "application/json")
                            .body(b"[]".to_vec())?,
                        cookies,
                    ));
                }
            },
        },
        router,
        &mut resp,
        &mut SubscriptionMap::None,
    )
    .await;

    match resp {
        Sender::Response(Some(resp)) => Ok((
            match serde_json::to_vec(&resp) {
                Ok(v) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(v)?,
                Err(_err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error serializing response: {}", _err);

                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "application/json")
                        .body(b"[]".to_vec())?
                }
            },
            cookies,
        )),
        _ => unreachable!(),
    }
}

pub fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    req: Request,
    cookies: CookieJar,
    router: Arc<Router<TCtx>>,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    #[cfg(feature = "tracing")]
    tracing::debug!("Accepting websocket connection");

    #[cfg(not(feature = "axum"))]
    return {
        println!("Sorry websocket are not supported on your platform yet!");
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![])?)
    };

    #[cfg(feature = "axum")]
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |req, mut socket| async move {
        use httpz::axum::axum::extract::RequestParts;

        let mut subscriptions = HashMap::new();
        let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);
        let mut req = RequestParts::new(req.into());

        loop {
            tokio::select! {
                biased; // Note: Order is important here
                msg = rx.recv() => {
                    match socket.send(Message::Text(match serde_json::to_string(&msg) {
                        Ok(v) => v,
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error serializing websocket message: {}", _err);

                            continue;
                        }
                    })).await {
                        Ok(_) => {}
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error sending websocket message: {}", _err);

                            continue;
                        }
                    }
                }
                msg = socket.next() => {
                    match msg {
                        Some(Ok(msg) )=> {
                           let res = match msg {
                                Message::Text(text) => serde_json::from_str::<Value>(&text),
                                Message::Binary(binary) => serde_json::from_slice(&binary),
                                Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                                    continue;
                                }
                                Message::Frame(_) => unreachable!(),
                            };

                            match res.and_then(|v| match v.is_array() {
                                    true => serde_json::from_value::<Vec<jsonrpc::Request>>(v),
                                    false => serde_json::from_value::<jsonrpc::Request>(v).map(|v| vec![v]),
                                }) {
                                Ok(reqs) => {
                                    for request in reqs {
                                        #[cfg(feature = "workers")]
                                        compile_error!("You can't have the 'axum' and 'workers' features enabled at the same time!");
                                        #[cfg(not(feature = "workers"))]
                                        {
                                            let ctx = match ctx_fn.exec(&mut req) {
                                                TCtxFuncResult::Value(v) => v,
                                                TCtxFuncResult::Future(v) => v.await,
                                            };


                                            handle_json_rpc(match ctx {
                                                Ok(v) => v,
                                                Err(_err) => {
                                                    #[cfg(feature = "tracing")]
                                                    tracing::error!("Error executing context function: {}", _err);

                                                    continue;
                                                }
                                            }, request, &router, &mut Sender::Channel(&mut tx),
                                            &mut SubscriptionMap::Ref(&mut subscriptions)).await;
                                        }
                                    }
                                },
                                Err(_err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("Error parsing websocket message: {}", _err);

                                    // TODO: Send report of error to frontend

                                    println!("Error in websocket: {}", _err);

                                    continue;
                                }
                            };
                        }
                        Some(Err(_err)) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error in websocket: {}", _err);

                            println!("Error in websocket: {}", _err);

                            // TODO: Send report of error to frontend

                            continue;
                        },
                        None => {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("Shutting down websocket connection");

                            // TODO: Send report of error to frontend

                            return;
                        },
                    }
                }
            }
        }
    })
}
