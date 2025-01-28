use std::{borrow::Borrow, collections::HashMap};

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{request::Parts, Method, Response, StatusCode},
    response::IntoResponse,
    routing::{on, MethodFilter},
    RequestExt, Router,
};
use rspc_procedure::{Procedure, Procedures};
use serde_json::Value;

use crate::{
    extractors::TCtxFunc,
    jsonrpc::{self, ProcedureKind, RequestId},
    jsonrpc_exec::{handle_json_rpc, Sender, SubscriptionMap},
};

pub fn endpoint<TCtx, TCtxFnMarker, TCtxFn, S>(
    procedures: impl Borrow<Procedures<TCtx>>,
    ctx_fn: TCtxFn,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TCtxFnMarker: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, S, TCtxFnMarker>,
{
    let procedures = procedures.borrow().clone();

    Router::<S>::new().route(
        "/:id",
        on(
            MethodFilter::GET.or(MethodFilter::POST),
            move |state: State<S>, req: axum::extract::Request<Body>| {
                let procedures = procedures.clone();

                async move {
                    match (req.method(), &req.uri().path()[1..]) {
                        (&Method::GET, "ws") => {
                            #[cfg(feature = "ws")]
                            {
                                let mut req = req;
                                return req
                                    .extract_parts::<axum::extract::ws::WebSocketUpgrade>()
                                    .await
                                    .unwrap() // TODO: error handling
                                    .on_upgrade(|socket| {
                                        handle_websocket(
                                            ctx_fn,
                                            socket,
                                            req.into_parts().0,
                                            procedures,
                                            state.0,
                                        )
                                    })
                                    .into_response();
                            }

                            #[cfg(not(feature = "ws"))]
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("[]")) // TODO: Better error message which frontend is actually setup to handle.
                                .unwrap()
                        }
                        (&Method::GET, _) => {
                            handle_http(ctx_fn, ProcedureKind::Query, req, &procedures, state.0)
                                .await
                                .into_response()
                        }
                        (&Method::POST, _) => {
                            handle_http(ctx_fn, ProcedureKind::Mutation, req, &procedures, state.0)
                                .await
                                .into_response()
                        }
                        _ => unreachable!(),
                    }
                }
            },
        ),
    )
}

async fn handle_http<TCtx, TCtxFn, TCtxFnMarker, TState>(
    ctx_fn: TCtxFn,
    kind: ProcedureKind,
    req: Request,
    procedures: &Procedures<TCtx>,
    state: TState,
) -> impl IntoResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
    TState: Send + Sync + 'static,
{
    let procedure_name = req.uri().path()[1..].to_string(); // Has to be allocated because `TCtxFn` takes ownership of `req`
    let (parts, body) = req.into_parts();
    let input = match parts.method {
        Method::GET => parts
            .uri
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
            .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(None as Option<Value>)),
        Method::POST => {
            // TODO: Limit body size?
            let body = to_bytes(body, usize::MAX).await.unwrap(); // TODO: error handling
            (!body.is_empty())
                .then(|| serde_json::from_slice(body.to_vec().as_slice()))
                .unwrap_or(Ok(None))
        }
        _ => unreachable!(),
    };

    let input = match input {
        Ok(input) => input,
        Err(_err) => {
            // #[cfg(feature = "tracing")]
            // tracing::error!("Error passing parameters to operation '{procedure_name}': {_err}");

            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Body::from(b"[]".as_slice()))
                .unwrap();
        }
    };

    // #[cfg(feature = "tracing")]
    // tracing::debug!("Executing operation '{procedure_name}' with params {input:?}");

    let mut resp = Sender::Response(None);

    let ctx = match ctx_fn.exec(parts, &state).await {
        Ok(ctx) => ctx,
        Err(_err) => {
            // #[cfg(feature = "tracing")]
            // tracing::error!("Error executing context function: {}", _err);

            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(b"[]".as_slice()))
                .unwrap();
        }
    };

    handle_json_rpc(
        ctx,
        jsonrpc::Request {
            jsonrpc: None,
            id: RequestId::Null,
            inner: match kind {
                ProcedureKind::Query => jsonrpc::RequestInner::Query {
                    path: procedure_name.to_string(), // TODO: Lifetime instead of allocate?
                    input,
                },
                ProcedureKind::Mutation => jsonrpc::RequestInner::Mutation {
                    path: procedure_name.to_string(), // TODO: Lifetime instead of allocate?
                    input,
                },
                ProcedureKind::Subscription => {
                    // #[cfg(feature = "tracing")]
                    // tracing::error!("Attempted to execute a subscription operation with HTTP");

                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "application/json")
                        .body(Body::from(b"[]".as_slice()))
                        .unwrap();
                }
            },
        },
        procedures,
        &mut resp,
        &mut SubscriptionMap::None,
    )
    .await;

    match resp {
        Sender::Response(Some(resp)) => match serde_json::to_vec(&resp) {
            Ok(v) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(v))
                .unwrap(),
            Err(_err) => {
                // #[cfg(feature = "tracing")]
                // tracing::error!("Error serializing response: {}", _err);

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Body::from(b"[]".as_slice()))
                    .unwrap()
            }
        },
        _ => unreachable!(),
    }
}

#[cfg(feature = "ws")]
async fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker, TState>(
    ctx_fn: TCtxFn,
    mut socket: axum::extract::ws::WebSocket,
    parts: Parts,
    procedures: Procedures<TCtx>,
    state: TState,
) where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
    TState: Send + Sync,
{
    use axum::extract::ws::Message;
    use futures::StreamExt;
    use tokio::sync::mpsc;

    // #[cfg(feature = "tracing")]
    // tracing::debug!("Accepting websocket connection");

    let mut subscriptions = HashMap::new();
    let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

    loop {
        tokio::select! {
            biased; // Note: Order is important here
            msg = rx.recv() => {
                match socket.send(Message::Text(match serde_json::to_string(&msg) {
                    Ok(v) => v,
                    Err(_err) => {
                        // #[cfg(feature = "tracing")]
                        // tracing::error!("Error serializing websocket message: {}", _err);

                        continue;
                    }
                })).await {
                    Ok(_) => {}
                    Err(_err) => {
                        // #[cfg(feature = "tracing")]
                        // tracing::error!("Error sending websocket message: {}", _err);

                        continue;
                    }
                }
            }
            msg = socket.next() => {
                match msg {
                    Some(Ok(msg)) => {
                       let res = match msg {
                            Message::Text(text) => serde_json::from_str::<Value>(&text),
                            Message::Binary(binary) => serde_json::from_slice(&binary),
                            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                                continue;
                            }
                        };

                        match res.and_then(|v| match v.is_array() {
                            true => serde_json::from_value::<Vec<jsonrpc::Request>>(v),
                            false => serde_json::from_value::<jsonrpc::Request>(v).map(|v| vec![v]),
                        }) {
                            Ok(reqs) => {
                                for request in reqs {
                                    let ctx = match ctx_fn.exec(parts.clone(), &state).await {
                                        Ok(ctx) => {
                                            ctx
                                        },
                                        Err(_err) => {

                                            // #[cfg(feature = "tracing")]
                                            // tracing::error!("Error executing context function: {}", _err);

                                            continue;
                                        }
                                    };

                                    handle_json_rpc(ctx, request, &procedures, &mut Sender::Channel(&mut tx),
                                    &mut SubscriptionMap::Ref(&mut subscriptions)).await;
                                }
                            },
                            Err(_err) => {
                                // #[cfg(feature = "tracing")]
                                // tracing::error!("Error parsing websocket message: {}", _err);

                                // TODO: Send report of error to frontend

                                continue;
                            }
                        };
                    }
                    Some(Err(_err)) => {
                        // #[cfg(feature = "tracing")]
                        // tracing::error!("Error in websocket: {}", _err);

                        // TODO: Send report of error to frontend

                        continue;
                    },
                    None => {
                        // #[cfg(feature = "tracing")]
                        // tracing::debug!("Shutting down websocket connection");

                        // TODO: Send report of error to frontend

                        return;
                    },
                }
            }
        }
    }
}
